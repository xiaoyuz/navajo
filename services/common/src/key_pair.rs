use secp256k1::{generate_keypair, Message, PublicKey, rand, Secp256k1, SecretKey};
use secp256k1::ecdsa::Signature;
use serde::{Deserialize, Serialize};
use ncrypto::algo::{base58, base64, sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    sec_key: SecretKey,
    pub_key: PublicKey,
}

impl KeyPair {

    pub fn new() -> Self {
        let (sec_key, pub_key) = generate_keypair(&mut rand::thread_rng());
        Self {
            sec_key, pub_key
        }
    }

    pub fn recover(bytes: &[u8]) -> KeyPair {
        let secp = Secp256k1::new();
        let sec_key = SecretKey::from_slice(bytes).unwrap();
        let pub_key = PublicKey::from_secret_key(&secp, &sec_key);
        Self {
            sec_key, pub_key
        }
    }

    pub fn gen_public_key(&self) -> String {
        let public_key = self.pub_key.serialize().to_vec();
        base64::encode_to_str(&public_key)
    }

    pub fn gen_address(&self) -> String {
        let public_key = self.pub_key.serialize().to_vec();
        let bytes = sha256::encode(&public_key);
        base58::encode(&bytes)
    }

    pub fn sign(&self, data: &str) -> String {
        let message = Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(data.as_bytes());
        let sig = self.sec_key.sign_ecdsa(message);
        let sign_bytes = sig.serialize_compact().to_vec();
        base64::encode_to_str(&sign_bytes)
    }
}

pub fn verify(src: &str, sign: &str, public_key: &str) -> bool {
    let src = src.as_bytes();
    let sign = base64::decode_from_str(sign);
    let public_key = base64::decode_from_str(public_key);

    let message = Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(src);

    let pub_key = PublicKey::from_slice(public_key.as_slice()).unwrap();
    let sign = Signature::from_compact(sign.as_slice()).unwrap();
    sign.verify(&message, &pub_key).is_ok()
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