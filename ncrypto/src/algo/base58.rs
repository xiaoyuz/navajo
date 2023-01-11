use base58::{FromBase58, ToBase58};

pub fn encode(bytes: &[u8]) -> String {
    bytes.to_base58()
}

pub fn decode(str: &str) -> Vec<u8> {
    str.from_base58().unwrap()
}