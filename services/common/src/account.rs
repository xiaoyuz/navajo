use serde::{Deserialize, Serialize};
use crate::key_pair::KeyPair;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Account {
    pub key_pair: KeyPair,
    pub address: String,
}

impl Account {

    pub fn new() -> Self {
        let key_pair = KeyPair::new();
        let address = key_pair.gen_address();
        Self { key_pair, address }
    }

    pub fn recover(bytes: &[u8]) -> Account {
        let key_pair = KeyPair::recover(bytes);
        let address = key_pair.gen_address();
        Account { key_pair, address }
    }

    pub fn sign_data(&self, data: &str) -> String {
        self.key_pair.sign(data)
    }
}