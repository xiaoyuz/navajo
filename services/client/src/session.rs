use std::sync::Arc;
use common::account::Account;
use nredis::RedisClient;

const SESSION_EXPIRE_SECONDS: u64 = 30 * 24 * 60 * 60; // 30 days

const CLIENT_DEVICE_ACCOUNT: &str = "client_device_account:";

const CLIENT_SESSION: &str = "client_session:";
const CLIENT_SECRET: &str = "client_secret:";
const CLIENT_DEVICE_ID: &str = "client_device_id:";

pub struct SessionClient {
    redis_client: Arc<RedisClient>,
}

impl SessionClient {
    pub fn new(redis_client: Arc<RedisClient>) -> Arc<Self> {
        Arc::new(Self { redis_client })
    }

    pub async fn get_device_account(&self, device_id: &str) -> Option<Account> {
        let json_str = self.redis_client.get(format!("{}{}", CLIENT_DEVICE_ACCOUNT, device_id).as_str()).await?;
        let account: Account = serde_json::from_str(&json_str).expect("json error");
        Some(account)
    }

    pub async fn set_device_account(&self, device_id: &str, account: &Account) {
        let key = format!("{}{}", CLIENT_DEVICE_ACCOUNT, device_id);
        let json_str = serde_json::to_string(account).expect("json error");
        self.redis_client.set_nx(&key, &json_str).await;
    }

    pub async fn get_session(&self, tcp_port: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_SESSION, tcp_port);
        self.redis_client.get(&key).await
    }

    pub async fn set_session(&self, tcp_port: &str, session: &str) {
        let key = format!("{}{}", CLIENT_SESSION, tcp_port);
        self.redis_client.set_ex(&key, session, SESSION_EXPIRE_SECONDS as usize).await;
    }

    pub async fn get_secret(&self, tcp_port: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_SECRET, tcp_port);
        self.redis_client.get(&key).await
    }

    pub async fn set_secret(&self, tcp_port: &str, secret: &str) {
        let key = format!("{}{}", CLIENT_SECRET, tcp_port);
        self.redis_client.set_ex(&key, secret, SESSION_EXPIRE_SECONDS as usize).await;
    }

    pub async fn get_device_id(&self, tcp_port: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_DEVICE_ID, tcp_port);
        self.redis_client.get(&key).await
    }

    pub async fn set_device_id(&self, tcp_port: &str, device_id: &str) {
        let key = format!("{}{}", CLIENT_DEVICE_ID, tcp_port);
        self.redis_client.set(&key, device_id).await;
    }
}

#[cfg(test)]
mod tests {
    use common::account::Account;
    use nredis::{RedisClient, RedisConfig};
    use crate::session::SessionClient;

    #[actix_rt::test]
    async fn test_session() {
        let redis_config = RedisConfig {
            host: "redis://127.0.0.1/".to_string(),
        };
        let rc = RedisClient::new(redis_config);
        let session_client1 = SessionClient::new(rc.clone());

        let account = Account::new();
        session_client1.set_device_account("1111111", &account).await;
        let res = session_client1.get_device_account("1111111").await;
        println!("{:?}", res);
    }
}