use std::collections::HashMap;
use mac_address::get_mac_address;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use common::errors::{NavajoError, NavajoResult};
use common::errors::NavajoErrorRepr::IoError;
use ncrypto::algo::{aes, sha256};

pub struct KeyDB {
    store: Mutex<InMemStore>,
}

impl KeyDB {
    pub async fn init() -> NavajoResult<Self> {
        let store = InMemStore::init().await?;
        Ok(Self {
            store: Mutex::new(store)
        })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.store.lock().await.get(key).await
    }

    pub async fn set(&self, key: &str, value: &str) -> Option<()> {
        self.store.lock().await.set(key, value).await
    }

    pub async fn remove(&self, key: &str) -> Option<()> {
        self.store.lock().await.remove(key).await
    }
}

struct InMemStore {
    kv: HashMap<String, String>,
    persist: Persist,
}

impl InMemStore {
    async fn init() -> NavajoResult<Self> {
        let kv = Default::default();
        let persist = Persist::init().await?;
        Ok(Self { kv, persist })
    }

    async fn get(&mut self, key: &str) -> Option<String> {
        match self.kv.get(key) {
            Some(x) => Some(x.to_string()),
            None => self.persist.get(key).await,
        }
    }

    async fn set(&mut self, key: &str, value: &str) -> Option<()> {
        self.persist.set(key, value).await?;
        self.kv.insert(key.to_string(), value.to_string())?;
        Some(())
    }

    async fn remove(&mut self, key: &str) -> Option<()> {
        self.persist.remove(key).await?;
        self.kv.remove(key);
        Some(())
    }
}

struct Persist {
    file: File,
}

impl Persist {
    async fn init() -> NavajoResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(false)
            .open(".navajo_ks")
            .await.map_err(|err| NavajoError::new(IoError(err)))?;

        Ok(Self { file })
    }

    async fn get(&mut self, key: &str) -> Option<String> {
        let res = self.read_file().await?;
        res.get(key).map(|x| x.to_string())
    }

    async fn set(&mut self, key: &str, value: &str) -> Option<()> {
        let mut res = self.read_file().await?;
        res.insert(key.to_string(), value.to_string());
        self.write_file(&res).await
    }

    async fn remove(&mut self, key: &str) -> Option<()> {
        let mut res = self.read_file().await?;
        res.remove(key);
        self.write_file(&res).await
    }

    async fn read_file(&mut self) -> Option<HashMap<String, String>> {
        let mut buf = Vec::new();
        self.file.read_to_end(&mut buf).await.ok()?;
        self.file.rewind().await.ok()?;

        let secret = key_store_secret()?;
        let decoded = aes::decode(&secret, &buf).unwrap_or(buf);

        let res: HashMap<String, String> = serde_json::from_slice(&decoded).unwrap_or_else(|_| Default::default());
        Some(res)
    }

    async fn write_file(&mut self, res: &HashMap<String, String>) -> Option<()> {
        let new_map = serde_json::to_vec(res).ok()?;

        let secret = key_store_secret()?;
        let buf = aes::encode(&secret, &new_map).ok()?;

        self.file.set_len(0).await.ok()?;
        self.file.write_all(&buf).await.ok()?;
        self.file.rewind().await.ok()?;
        Some(())
    }
}

fn key_store_secret() -> Option<Vec<u8>> {
    let mac_addr = get_mac_address().ok()??;
    Some(sha256::encode(&mac_addr.bytes()))
}