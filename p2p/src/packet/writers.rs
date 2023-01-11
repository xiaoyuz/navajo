use ncrypto::algo::{aes, base64};
use crate::packet::p2p_packet::PacketContent;

pub trait Writer {
    fn process(&self, data: &str, params: &[&str]) -> Option<String>;

    fn successor(&self) -> Option<Box<dyn Writer>>;

    fn successor_process(&self, data: &str, params: &[&str]) -> Option<String> {
        self.successor()?.process(data, params)
    }
}

pub struct BasicWriter;

impl Writer for BasicWriter {
    fn process(&self, data: &str, _params: &[&str]) -> Option<String> {
        Some(format!("<{}>", data))
    }

    fn successor(&self) -> Option<Box<dyn Writer>> {
        None
    }
}

pub struct CryptoWriter;

impl Writer for CryptoWriter {

    fn process(&self, data: &str, params: &[&str]) -> Option<String> {
        if params.len() < 2 {
            return None;
        }
        let session = params[0];
        let secret = params[1];

        let secret = secret.as_bytes();
        let data = data.as_bytes();

        let encrypted_data = aes::encode(secret, data).ok()?;

        let encrypted_data = base64::encode_to_str(&encrypted_data);
        let content = PacketContent {
            data: encrypted_data,
            session: String::from(session),
        };
        let json = serde_json::to_string(&content).unwrap();
        let encoded = base64::encode_to_str(json.as_bytes());
        self.successor_process(&encoded, &[])
    }

    fn successor(&self) -> Option<Box<dyn Writer>> {
        Some(Box::new(BasicWriter))
    }
}

pub struct MessageWriter;

impl Writer for MessageWriter {
    fn process(&self, data: &str, params: &[&str]) -> Option<String> {
        let res = base64::encode_to_str(data.as_bytes());
        self.successor_process(&res, params)
    }

    fn successor(&self) -> Option<Box<dyn Writer>> {
        Some(Box::new(CryptoWriter))
    }
}

