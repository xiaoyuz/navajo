use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::message::Message::PingMessage;

pub const TEXT_TYPE: MessageType = 0;

pub const MESSAGE_TYPE_PING: MessageType = 0;
pub const MESSAGE_TYPE_CHAT_MESSAGE: MessageType = 1;

type MessageType = u8;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct P2PMessage {
    pub message_type: u8,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NetworkData {
    pub ip: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommonInfo {
    pub time_ms: u128,
    pub request_id: String,
    pub response_id: String,
}

impl Default for CommonInfo {
    fn default() -> Self {
        CommonInfo {
            time_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            request_id: Uuid::new_v4().to_string(),
            response_id: String::from(""),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    PingMessage {
        address: String,
        device_id: String,
    },
    ChatInfoMessage {
        common_info: CommonInfo,
        from_address: String,
        to_address: String,
        info_type: u8,
        content: String,
    },
}

impl From<&Message> for String {
    fn from(value: &Message) -> Self {
        serde_json::to_string(value).unwrap()
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }
}

impl From<&[u8]> for Message {
    fn from(value: &[u8]) -> Self {
        serde_json::from_slice(value).unwrap()
    }
}

impl From<Vec<u8>> for Message {
    fn from(value: Vec<u8>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

impl From<&Message> for Vec<u8> {
    fn from(value: &Message) -> Self {
        serde_json::to_vec(value).unwrap()
    }
}

impl From<&Message> for P2PMessage {
    fn from(value: &Message) -> Self {
        let message_type = match value {
            PingMessage { .. } => MESSAGE_TYPE_PING,
            _ => MESSAGE_TYPE_CHAT_MESSAGE,
        };
        P2PMessage {
            message_type,
            data: value.into()
        }
    }
}

impl From<&P2PMessage> for Message {
    fn from(value: &P2PMessage) -> Self {
        value.data.as_str().into()
    }
}

impl From<&[u8]> for P2PMessage {
    fn from(value: &[u8]) -> Self {
        serde_json::from_slice(value).unwrap()
    }
}

impl From<Vec<u8>> for P2PMessage {
    fn from(value: Vec<u8>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

impl From<&P2PMessage> for Vec<u8> {
    fn from(value: &P2PMessage) -> Self {
        serde_json::to_vec(value).unwrap()
    }
}

impl From<&str> for P2PMessage {
    fn from(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }
}

impl From<&P2PMessage> for String {
    fn from(value: &P2PMessage) -> Self {
        serde_json::to_string(value).unwrap()
    }
}