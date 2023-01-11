use std::{error, fmt};
use std::fmt::Formatter;
use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::AeadMut;
use aes_gcm::aead::generic_array::GenericArray;

#[derive(Debug, Clone)]
pub enum AESError {
    EncryptError,
    DecryptError,
}

impl fmt::Display for AESError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AESError::EncryptError => write!(f, "{}", "AES encrypt error"),
            AESError::DecryptError => write!(f, "{}", "AES decrypt error"),
        }
    }
}

impl error::Error for AESError {}

type AESResult<T> = Result<T, AESError>;

pub fn encode(key: &[u8], data: &[u8]) -> AESResult<Vec<u8>> {
    let nonce = &[0u8; 12];
    let key = GenericArray::from_slice(key);
    let mut cipher = Aes256Gcm::new(&key);
    let nonce = GenericArray::from_slice(nonce);
    cipher.encrypt(&nonce, data).map_err(|_| { AESError::EncryptError })
}

pub fn decode(key: &[u8], data: &[u8]) -> AESResult<Vec<u8>> {
    let nonce = &[0u8; 12];
    let key = GenericArray::from_slice(key);
    let mut cipher = Aes256Gcm::new(&key);
    let nonce = GenericArray::from_slice(nonce);
    cipher.decrypt(&nonce, data).map_err(|_| { AESError::DecryptError })
}