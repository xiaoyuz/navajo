use serde::{Deserialize, Serialize};
use crate::key_pair::KeyPair;

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn recover(mnemonic: &str) -> Account {
        let key_pair = KeyPair::recover(mnemonic);
        let address = key_pair.gen_address();
        Account { key_pair, address }
    }

    pub fn sign_data(&self, data: &str) -> String {
        self.key_pair.sign(data)
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Account {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl From<&Account> for String {
    fn from(value: &Account) -> Self {
        serde_json::to_string(value).unwrap()
    }
}