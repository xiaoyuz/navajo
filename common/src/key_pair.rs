use serde::{Deserialize, Serialize};
use ncrypto::algo::{base58, base64, ecdsa, sha256};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KeyPair {
    bytes: Vec<u8>,
}

impl KeyPair {

    pub fn new() -> Self {
        Self {
            bytes: ecdsa::gen_key_pair()
        }
    }

    pub fn recover(bytes: &[u8]) -> KeyPair {
        KeyPair {
            bytes: bytes.to_vec()
        }
    }

    pub fn gen_public_key(&self) -> String {
        let public_key = ecdsa::gen_public_key(&self.bytes);
        base64::encode_to_str(&public_key)
    }

    pub fn gen_address(&self) -> String {
        let public_key = ecdsa::gen_public_key(&self.bytes);
        let bytes = sha256::encode(&public_key);
        base58::encode(&bytes)
    }

    pub fn sign(&self, data: &str) -> String {
        let sign_bytes = ecdsa::sign(&self.bytes, data);
        base64::encode_to_str(&sign_bytes)
    }
}

pub fn verify(src: &str, sign: &str, public_key: &str) -> bool {
    let src = src.as_bytes();
    let sign = base64::decode_from_str(sign);
    let public_key = base64::decode_from_str(public_key);
    ecdsa::verify(src, &sign, &public_key)
}

#[cfg(test)]
mod tests {
    use crate::key_pair::{KeyPair, verify};

    #[test]
    fn test_key_pair() {
        let key_pair = KeyPair::new();
        let my_public_key = key_pair.gen_public_key();
        println!("{:?}", my_public_key);

        let data = "This is an apple.";
        let sign = key_pair.sign(data);
        println!("{:?}", sign);

        let res = verify(data, &sign, &my_public_key);
        println!("{:?}", res);
    }
}