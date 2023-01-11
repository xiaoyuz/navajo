use std::sync::Arc;
use ncrypto::algo::base64::{decode_from_str, encode_to_str};
use nredis::RedisClient;
use p2p::message::Message;

const CHAT_MESSAGE_EXPIRE_SECONDS: u64 = 180 * 24 * 60 * 60; // 180 days

const KEY_MESSAGE_QUEUE_ADDRESS: &str = "key_message_queue_address:";

const STORE_SPLITER: &str = ">";

pub struct QueueManager {
    redis_client: Arc<RedisClient>,
}

impl QueueManager {
    pub fn new(redis_client: Arc<RedisClient>) -> Arc<Self> {
        Arc::new(Self { redis_client })
    }

    pub async fn acquire_queue(&self, address: &str) -> Option<Vec<Message>> {
        let key = format!("{}{}", KEY_MESSAGE_QUEUE_ADDRESS, address);
        let value = self.redis_client.get(&key).await?;
        let splits: Vec<&str> = value.split(STORE_SPLITER).collect();
        let messages: Vec<Message> = splits.iter().map(|item| {
            decode_from_str(item).into()
        }).collect();
        Some(messages)
    }

    pub async fn add_queue(&self, message: &Message) {
        let json: Vec<u8> = message.into();
        if let Message::ChatInfoMessage { to_address, .. } = message {
            let key = format!("{}{}", KEY_MESSAGE_QUEUE_ADDRESS, to_address);
            let value = encode_to_str(&json);
            let stored_value: String = self.redis_client.get(&key).await.map_or_else(
                || value.clone(),
                |current| format!("{}{}{}", current, STORE_SPLITER, value.clone())
            );
            self.redis_client.set_ex(&key, &stored_value, CHAT_MESSAGE_EXPIRE_SECONDS as usize).await
        }
    }

    pub async fn remove(&self, address: &str) {
        let key = format!("{}{}", KEY_MESSAGE_QUEUE_ADDRESS, address);
        self.redis_client.remove(&key).await
    }
}