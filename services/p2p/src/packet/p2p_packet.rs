use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default)]
pub struct P2PPacket {
    pub content: String,
    pub with_head: bool,
    pub with_tail: bool,
}

impl P2PPacket {

    pub fn concat(&self, other: &P2PPacket) -> P2PPacket {
        if self.with_tail {
            return self.clone();
        }
        if other.with_head {
            return other.clone();
        }
        let mut content = self.content.to_owned();
        content.push_str(&other.content);

        P2PPacket {
            content,
            with_head: self.with_head,
            with_tail: other.with_tail,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PacketContent {
    pub data: String,
    pub session: String,
}

impl From<&str> for PacketContent {
    fn from(value: &str) -> Self {
        serde_json::from_str(value).unwrap()
    }
}

impl From<&PacketContent> for String {
    fn from(value: &PacketContent) -> Self {
        serde_json::to_string(value).unwrap()
    }
}