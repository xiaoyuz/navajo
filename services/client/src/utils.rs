use mac_address::get_mac_address;
use uuid::Uuid;
use common::errors::{MAC_ADDR_ERROR, NavajoError, NavajoResult};
use ncrypto::algo::{base64, sha256};

pub fn key_store_secret() -> Option<String> {
    let mac_addr = get_mac_address().ok()??;
    let bs = sha256::encode(&mac_addr.bytes());
    Some(base64::encode_to_str(bs.as_slice()))
}