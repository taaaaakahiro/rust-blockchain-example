use serde::{Deserialize, Serialize};

pub const PROTOCOL_NAME: &str = "blockchain-rs_protocol";
pub const MY_VERSION: &str = "0.1.0";

pub const MSG_ADD: usize = 0;
pub const MSG_REMOVE: usize = 1;
pub const MSG_CORE_LIST: usize = 2;
pub const MSG_REQUEST_CORE_LIST: usize = 3;
pub const MSG_PING: usize = 4;
pub const MSG_ADD_AS_EDGE: usize = 5;
pub const MSG_REMOVE_EDGE: usize = 6;
pub const MSG_NEW_TRANSACTION: usize = 7;
pub const MSG_NEW_BLOCK: usize = 8;
pub const MSG_NEW_BLOCK_TO_ALL: usize = 9;
pub const MSG_REQUEST_FULL_CHAIN: usize = 10;
pub const RSP_FULL_CHAIN: usize = 11;
pub const MSG_ENHANCED: usize = 12;
pub const MSG_UNLOCKED: usize = 13;
pub const MSG_SENDMSGALLPEAR: usize = 14;

pub const ERR_PROTOCOL_UNMATCH: usize = 0;
pub const ERR_VERSION_UNMATCH: usize = 1;
pub const OK_WITH_PAYLOAD: usize = 2;
pub const OK_WITHOUT_PAYLOAD: usize = 3;

pub const ERROR: usize = 0;
pub const OK: usize = 1;
pub const NONE: usize = 0;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageManager {
    pub protocol: String,
    pub version: String,
    pub msg_type: usize,
    pub ip: String,
    pub port: String,
    pub payload: String,
}

pub fn build(msg_type: usize, ip: &str, port: &str, payload: &str) -> String {
    //! It builds a message in String format with a given msg_type, ip, port and payload.    

    let mm = MessageManager {
        protocol: PROTOCOL_NAME.to_string(),
        version: MY_VERSION.to_string(),
        msg_type: msg_type,
        ip: ip.to_string(),
        port: port.to_string(),
        payload: payload.to_string(),
    };

    serde_json::to_string(&mm).unwrap()
}

pub fn parse(msg: &str) -> Vec<String> {
    //! It parses a message build by the build function.

    let mm: MessageManager = serde_json::from_str(msg).unwrap();

    println!(
        "parsed message: {} {} {} {} {}",
        mm.version, mm.msg_type, mm.ip, mm.port, mm.payload
    );
    println!("payload: {}", mm.payload);

    let mut res: Vec<String> = Vec::new();
    if mm.protocol != PROTOCOL_NAME {
        res = vec![
            ERROR.to_string(),
            ERR_PROTOCOL_UNMATCH.to_string(),
            NONE.to_string(),
            NONE.to_string(),
            NONE.to_string(),
            NONE.to_string(),
        ];
    } else if mm.version != MY_VERSION {
        res = vec![
            ERROR.to_string(),
            ERR_VERSION_UNMATCH.to_string(),
            NONE.to_string(),
            NONE.to_string(),
            NONE.to_string(),
            NONE.to_string(),
        ];
    } else if mm.msg_type == MSG_CORE_LIST
        || mm.msg_type == MSG_NEW_TRANSACTION
        || mm.msg_type == MSG_NEW_BLOCK
        || mm.msg_type == MSG_NEW_BLOCK_TO_ALL
        || mm.msg_type == RSP_FULL_CHAIN
        || mm.msg_type == MSG_ENHANCED
    {
        res = vec![
            OK.to_string(),
            OK_WITH_PAYLOAD.to_string(),
            mm.msg_type.to_string(),
            mm.ip.to_string(),
            mm.port.to_string(),
            mm.payload.to_string(),
        ];
    } else {
        res = vec![
            OK.to_string(),
            OK_WITHOUT_PAYLOAD.to_string(),
            mm.msg_type.to_string(),
            mm.ip.to_string(),
            mm.port.to_string(),
            NONE.to_string(),
        ];
    }
    res
}

pub fn classify_msg(msg: &str) -> bool {
    let res = parse(msg);
    let msg_type = &res[2];
    let msg_type_num: usize = msg_type.parse().unwrap();
    if msg_type_num <= MSG_REMOVE_EDGE {
        return true;
    }
    return false;
}
