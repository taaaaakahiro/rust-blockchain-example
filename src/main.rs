mod client_server_core;

use std::thread;
use std::time::Duration;

extern crate rand;
use rand::Rng;

use crate::client_server_core::block_chain::UTXOManager;
use std::sync::mpsc;

/// Wallet for edge nodes. It manages coins to be sent and received to/from other edge nodes.
pub struct Wallet {
    km: client_server_core::block_chain::keymanager::KeyManager,
    um: UTXOManager,
    client_core: client_server_core::ClientCore,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
}

impl Wallet {
    pub fn create(my_ip: &str, my_port: &str, my_core_ip: &str, my_core_port: &str) -> Wallet {
        let mut rng = rand::thread_rng();
        let rand_num = rng.gen::<usize>();
        let km = client_server_core::block_chain::keymanager::KeyManager::create(rand_num);
        let my_address = km.my_address();
        let um = UTXOManager::create(&my_address);

        let (tx, rx) = mpsc::channel::<String>();

        Wallet {
            km: km,
            um: um,
            client_core: client_server_core::ClientCore::create(
                my_ip,
                my_port,
                my_core_ip,
                my_core_port,
            ),
            tx: tx,
            rx: rx,
        }
    }

    pub fn create_coinbase_transaction(&mut self) {
        let my_address = self.km.my_address();
        let mut um = UTXOManager::create(&my_address);

        let t1 = client_server_core::block_chain::Transaction::create_coinbase_transaction(
            &self.km.my_address(),
            30,
        );
        let t2 = client_server_core::block_chain::Transaction::create_coinbase_transaction(
            &self.km.my_address(),
            30,
        );
        let t3 = client_server_core::block_chain::Transaction::create_coinbase_transaction(
            &self.km.my_address(),
            30,
        );

        let mut transactions: Vec<client_server_core::block_chain::Transaction> = Vec::new();
        transactions.push(t1);
        transactions.push(t2);
        transactions.push(t3);

        um.extract_utxo(&transactions);
        println!("my_address: {}", my_address);
        println!("my_balance: {}", um.my_balance);
        self.um = um;
    }

    pub fn update_wallet(&mut self) {
        self.update_block_chain();
        if let Ok(msg) = self.rx.try_recv() {
            self.client_core.bc = serde_json::from_str(&msg).unwrap();
            let tx = self.client_core.bc.get_stored_transactions_from_bc();
            self.um.extract_utxo(&tx);
            println!("my_address: {}", self.km.my_address());
            println!("my_balance: {}", self.um.my_balance);
        }
    }

    fn update_block_chain(&mut self) {
        self.client_core.send_req_full_chain_to_my_core_node();
    }

    pub fn start(&mut self) {
        self.client_core.start(&self.tx);
    }

    pub fn show_my_block_chain(&self) {
        println!("print current blockchain:");
        self.client_core.bc.print();
    }

    pub fn send(&mut self, recipient: &str, amount: i64, sendfee: i64) {
        println!("my_balance: {}", self.um.my_balance);

        if recipient.len() == 0 {
            println!("Please enter the recipient address!");
            return;
        } else if amount <= 0 {
            println!("Total amount is no less than 0");
            return;
        } else if sendfee <= 0 {
            println!("Fee is no less than 0");
            return;
        }

        let utxo_len = self.um.utxo_txs.len();
        println!("utxo_len: {}", utxo_len);

        if utxo_len > 0 {
            println!("Sending {} to receiver {}", amount, recipient);
        } else {
            println!("Short of coin. Not enough coin to be sent.");
            return;
        }

        let (utxo, idx) = self.um.get_utxo_tx(0);

        let mut t = client_server_core::block_chain::Transaction::create();

        let t_in = client_server_core::block_chain::TransactionInput::create(utxo.clone(), *idx);
        let t_out = client_server_core::block_chain::TransactionOutput::create(recipient, amount);

        t.inputs.push(t_in);
        t.outputs.push(t_out);

        let mut counter = 1;
        while t.is_enough_inputs(sendfee) == false {
            let (new_uxto, new_idx) = self.um.get_utxo_tx(counter);
            t.inputs
                .push(client_server_core::block_chain::TransactionInput::create(
                    new_uxto.clone(),
                    *new_idx,
                ));
            counter += 1;
            if counter >= utxo_len {
                println!("Short of Coin. Not enough coin to be sent");
                break;
            }
        }

        if t.is_enough_inputs(sendfee) == true {
            let change = t.compute_change(sendfee);
            println!("change: {}", change);
            let t_out = client_server_core::block_chain::TransactionOutput::create(
                &self.km.my_address(),
                change,
            );
            t.outputs.push(t_out);
            t.signature = "".to_string();
            let to_be_signed = serde_json::to_string(&t).unwrap();
            let signed = self.km.compute_digital_signature(&to_be_signed);
            t.signature = signed;
            let tx_string = serde_json::to_string(&t).unwrap();
            self.client_core.send_message_to_my_core_node(
                client_server_core::connection_manager::message_manager::MSG_NEW_TRANSACTION,
                &tx_string,
            );
            println!("signed new_tx: {}", tx_string);
            self.um.put_utxo_tx(&t);

            let mut to_be_deleted = 0;
            let mut del_list_idx = Vec::new();
            while to_be_deleted < counter {
                let del_tx = self.um.get_utxo_tx(to_be_deleted);
                del_list_idx.push(del_tx.1);
                to_be_deleted += 1;
            }
            for idx in del_list_idx.iter() {
                self.um.remove_utxo_tx(*idx);
            }
        }
        println!("my updated balance: {}", self.um.my_balance);
    }
}

fn start_server1() {
    println!("start server1");
    let ip = "127.0.0.1";
    let port = "8880";

    let mut my_p2p_server = client_server_core::ServerCore::create(
        &ip.to_string(),
        &port.to_string(),
        &"".to_string(),
        &"".to_string(),
    );
    my_p2p_server.start();

    loop {}
}

fn start_server2() {
    println!("start server2");
    let ip = "127.0.0.1";
    let port = "50090";

    let ip_core = "127.0.0.1";
    let port_core = "8880";

    let mut my_p2p_server = client_server_core::ServerCore::create(
        &ip.to_string(),
        &port.to_string(),
        &ip_core.to_string(),
        &port_core.to_string(),
    );
    my_p2p_server.start();
    my_p2p_server.join_network();

    loop {}
}

fn start_client1() {
    let my_ip = "127.0.0.1";
    let my_port = "50092";

    let my_core_ip = "127.0.0.1";
    let my_core_port = "50090";

    let mut wallet = Wallet::create(my_ip, my_port, my_core_ip, my_core_port);

    println!("wallet start");
    wallet.start();

    loop {
        wallet.update_wallet();
        thread::sleep(Duration::from_millis(50000));
    }
}

fn start_client2() {
    let my_ip = "127.0.0.1";
    let my_port = "50093";

    let my_core_ip = "127.0.0.1";
    let my_core_port = "8880";

    let mut wallet = Wallet::create(my_ip, my_port, my_core_ip, my_core_port);
    wallet.create_coinbase_transaction();

    println!("wallet start");

    wallet.start();

    let recipient = "untrusted comment: minisign public key: AE8BF9CAD01429A5\nRWSlKRTQyvmLrnX0rvRivOpEWl8zN2+0eEtmLDw8Vsq8Snudkyf4DYMZ\n";

    wallet.send(recipient, 30, 5); // send 30 coins with 5 fee to the recipient

    loop {
        wallet.update_wallet();
        thread::sleep(Duration::from_millis(50000));
    }
}

fn main() {
    // start_server1();
    // start_server2();

    // start_client1();
    start_client2();
}
