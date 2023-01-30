use std::sync::Arc;
use redis::{AsyncCommands, Client, RedisResult};
use redis::aio::Connection;
use crate::db::RedisConfig;

pub struct RedisClient {
    rc: Client,
}

impl RedisClient {
    pub fn new(redis_config: RedisConfig) -> Arc<Self> {
        let host = redis_config.host;
        let rc = Client::open(host).expect("failed to connect redis");
        Arc::new(Self { rc })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut con = self.con().await;
        let res: RedisResult<String> = con.get(key).await;
        if res.is_err() {
            return None;
        }
        Some(res.unwrap())
    }

    pub async fn set(&self, key: &str, value: &str) {
        let mut con = self.con().await;
        con.set(key, value).await.expect("redis error")
    }

    pub async fn set_ex(&self, key: &str, value: &str, secs: usize) {
        let mut con = self.con().await;
        con.set_ex(key, value, secs).await.expect("redis error")
    }

    pub async fn set_nx(&self, key: &str, value: &str) {
        let mut con = self.con().await;
        con.set_nx(key, value).await.expect("redis error")
    }

    pub async fn remove(&self, key: &str) {
        let mut con = self.con().await;
        con.del(key).await.expect("redis error")
    }

    async fn con(&self) -> Connection {
        self.rc.get_async_connection().await.unwrap()
    }
}