use std::rc::Rc;
use ncrypto::algo::{aes, base64};
use crate::message::P2PMessage;
use crate::packet::p2p_packet::{P2PPacket, PacketContent};

pub trait Reader {
    fn process(&mut self, data: &str) -> Option<P2PMessage>;

    fn successor(&self) -> Option<Box<dyn Reader>>;

    fn successor_process(&self, data: &str) -> Option<P2PMessage> {
        self.successor()?.process(data)
    }
}

pub struct MessageReader;

impl Reader for MessageReader {
    fn process(&mut self, data: &str) -> Option<P2PMessage> {
        let res = base64::decode_from_str(data);
        let res = String::from_utf8(res).unwrap();
        serde_json::from_str(&res).ok()
    }

    fn successor(&self) -> Option<Box<dyn Reader>> {
        None
    }
}

pub struct CryptoReader {
    query_secret: Rc<dyn Fn(String) -> Option<String>>,
}

impl Reader for CryptoReader {
    fn process(&mut self, data: &str) -> Option<P2PMessage> {
        let data = base64::decode_from_str(data);
        let data = String::from_utf8(data).unwrap();
        let packet_content: PacketContent = serde_json::from_str(&data).unwrap();
        let session = packet_content.session;
        let secret = &self.query_secret;
        let secret = secret(session)?;
        let data = base64::decode_from_str(&packet_content.data);
        let content = aes::decode(secret.as_bytes(), &data).ok()?;

        let content = String::from_utf8(content).unwrap();
        self.successor_process(&content)
    }

    fn successor(&self) -> Option<Box<dyn Reader>> {
        Some(Box::new(MessageReader))
    }
}

pub struct BasicReader {
    query_secret: Rc<dyn Fn(String) -> Option<String>>,
    temp_packet: Option<P2PPacket>,
}

impl BasicReader {
    pub fn new(query_secret: Box<dyn Fn(String) -> Option<String>>) -> Self {
        Self {
            query_secret: Rc::new(query_secret),
            temp_packet: None
        }
    }
}

impl Reader for BasicReader {
    fn process(&mut self, data: &str) -> Option<P2PMessage> {
        let packets = packets_from_string(data);
        for packet in packets {
            if packet.with_head && packet.with_tail {
                return self.successor_process(&packet.content);
            } else {
                if let Some(_) = self.temp_packet {
                    self.temp_packet = self.temp_packet.as_mut().map(|x| { x.concat(&packet) });
                } else {
                    self.temp_packet = Some(packet);
                }
                let temp = self.temp_packet.as_ref().unwrap();
                if temp.with_tail && temp.with_head {
                    let res = self.successor_process(&temp.content);
                    self.temp_packet = None;
                    return res;
                }
            }
        }
        None
    }

    fn successor(&self) -> Option<Box<dyn Reader>> {
        Some(Box::new(CryptoReader { query_secret: self.query_secret.clone() }))
    }
}

fn packets_from_string(str: &str) -> Vec<P2PPacket> {
    let mut packets: Vec<P2PPacket> = vec![];
    let mut res: Option<String> = None;
    let mut packet: Option<P2PPacket> = None;
    for char in str.chars() {
        packet = packet.or(Some(Default::default()));
        res = res.or(Some(String::from("")));
        if char == '<' {
            res = Some(String::from(""));
            packet = packet.map(|mut x| { x.with_head = true; x });
        } else if char == '>' {
            packet = packet.map(|mut x| {
                x.with_tail = true;
                x.content = res.unwrap();
                x
            });
            packets.push(packet.unwrap());
            packet = None;
            res = None;
        } else {
            res = res.map(|mut x| { x.push(char); x });
        }
    }
    if let Some(_) = packet {
        packet = packet.map(|mut x| { x.content = res.unwrap(); x });
        packets.push(packet.unwrap());
    }
    packets
}