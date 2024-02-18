//! It manages the private_key and public_key, and it computes a digital signature using those keys.

extern crate minisign;
use minisign::{KeyPair, PublicKeyBox, SecretKeyBox, SignatureBox};
use std::io::Cursor;

extern crate rand;
use rand::seq::SliceRandom;

const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub struct KeyManager {
    pub private_key_str: String,
    pub public_key_str: String,
    password: String,
}

impl KeyManager {
    pub fn create(random_num: usize) -> KeyManager {
        //! create private_key and public_key with a random number

        let random_pass = gen_ascii_chars(random_num);
        let KeyPair { pk, sk } =
            KeyPair::generate_encrypted_keypair(Some(random_pass.clone())).unwrap();

        let private_key_str = sk.to_box(None).unwrap().to_string();
        let public_key_str = pk.to_box().unwrap().to_string();
        println!("private key: {:#?}", private_key_str);
        println!("public key: {:#?}", public_key_str);

        KeyManager {
            private_key_str: private_key_str,
            public_key_str: public_key_str,
            password: random_pass.clone(),
        }
    }

    pub fn my_address(&self) -> String {
        //! return the address as a public_key

        self.public_key_str.clone()
    }

    pub fn compute_digital_signature(&self, message: &str) -> String {
        let sk_box = SecretKeyBox::from_string(&self.private_key_str).unwrap();
        let sk = sk_box.into_secret_key(Some(self.password.clone())).unwrap();

        let msg_reader = Cursor::new(message);
        let signature_box = minisign::sign(None, &sk, msg_reader, None, None).unwrap();

        signature_box.into_string()
    }

    pub fn verify_signature(
        &self,
        message: &str,
        signature_box_str: &str,
        sender_public_key_box_str: &str,
    ) -> bool {
        let signature_box = SignatureBox::from_string(signature_box_str).unwrap();

        let pk_box = PublicKeyBox::from_string(&sender_public_key_box_str).unwrap();
        let pk = pk_box.into_public_key().unwrap();

        let msg_reader = Cursor::new(message);
        let verified = minisign::verify(&pk, &signature_box, msg_reader, true, false, false);

        let flag: bool;
        match verified {
            Ok(()) => flag = true,
            Err(_) => flag = false,
        };
        return flag;
    }

    pub fn export_key_pair(&mut self, key_data: &str, pass_phrase: &str) -> (String, String) {
        return (self.private_key_str.clone(), self.public_key_str.clone());
    }

    pub fn import_key_pair(&mut self, private_key_str: &str, public_key_str: &str) {
        self.private_key_str = private_key_str.to_string();
        self.public_key_str = public_key_str.to_string();
    }

    pub fn clone(&self) -> KeyManager {
        KeyManager {
            private_key_str: self.private_key_str.clone(),
            public_key_str: self.public_key_str.clone(),
            password: self.password.clone(),
        }
    }
}

fn gen_ascii_chars(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect(),
    )
    .unwrap()
}

pub fn run() {
    let km = KeyManager::create(40);

    let my_address = km.my_address();
    println!("my_address: {}", my_address);

    let msg = "The first message";
    let signature = km.compute_digital_signature(&msg);

    let flag = km.verify_signature(msg, &signature, &km.public_key_str);
    println!("verify suffcess {}", flag);

    let msg2 = "The second message";
    let flag2 = km.verify_signature(msg2, &signature, &km.public_key_str);
    println!("verify suffcess {}", flag2);
}
