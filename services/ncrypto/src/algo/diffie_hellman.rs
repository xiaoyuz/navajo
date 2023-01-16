use rand_core::{OsRng};
use x25519_dalek::{EphemeralSecret, PublicKey};
use crate::algo::base64::{decode_from_str, encode_to_str};

pub struct DiffieHellman {
    pub public_key: PublicKey,
    pub secret: EphemeralSecret,
}

impl DiffieHellman {
    pub fn new() -> Self {
        let secret = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&secret);
        Self { public_key, secret }
    }

    pub fn compute_shared_secret(self, other_public_key: &PublicKey) -> Vec<u8> {
        self.secret.diffie_hellman(other_public_key).as_bytes().to_vec()
    }

    pub fn compute_shared_secret_from_str(self, other_public_key_str: &str) -> Vec<u8> {
        let other_public_key = decode_from_str(other_public_key_str);
        let other_public_key = other_public_key.as_ptr() as *const [u8; 32];
        unsafe {
            let other_public_key = PublicKey::from(*other_public_key);
            self.secret.diffie_hellman(&other_public_key).as_bytes().to_vec()
        }
    }

    pub fn public_key_to_str(&self) -> String {
        let bytes = self.public_key.as_bytes();
        encode_to_str(bytes)
    }
}