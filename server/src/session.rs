use std::sync::Arc;
use serde::{Deserialize, Serialize};
use nredis::RedisClient;

const SESSION_EXPIRE_SECONDS: u64 = 30 * 24 * 60 * 60; // 30 days

const SESSION_INFO: &str = "session_info:";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionInfo {
    pub device_id: String,
    pub address: String,
    pub secret: String,
}

pub struct SessionClient {
    redis_client: Arc<RedisClient>,
}

impl SessionClient {
    pub fn new(redis_client: Arc<RedisClient>) -> Arc<Self> {
        Arc::new(Self { redis_client })
    }

    pub async fn get_session(&self, session: &str) -> Option<SessionInfo> {
        let key = format!("{}{}", SESSION_INFO, session);
        let json_str = self.redis_client.get(&key).await?;
        let session_info: SessionInfo = serde_json::from_str(&json_str).expect("json error");
        return Some(session_info);
    }

    pub async fn set_session(&self, session: &str, session_info: &SessionInfo) {
        let key = format!("{}{}", SESSION_INFO, session);
        let json_str = serde_json::to_string(session_info).expect("json error");
        self.redis_client.set_ex(&key, &json_str, SESSION_EXPIRE_SECONDS as usize).await;
    }
}

#[cfg(test)]
mod tests {
    use nredis::{RedisClient, RedisConfig};
    use crate::session::{SessionClient, SessionInfo};

    #[actix_rt::test]
    async fn test_session() {
        let redis_config = RedisConfig {
            host: "redis://127.0.0.1/"
        };
        let rc = RedisClient::new(Box::new(redis_config));
        let session_client1 = SessionClient::new(rc.clone());

        let session_info = SessionInfo {
            device_id: "123".to_string(),
            address: "456".to_string(),
            secret: "121".to_string()
        };
        session_client1.set_session("1111111", &session_info).await;
        let res = session_client1.get_session("1111111").await;
        println!("{:?}", res);
    }
}