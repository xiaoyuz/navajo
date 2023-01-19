use ncrypto::algo::aes;
use ncrypto::algo::base64::decode_from_str;
use crate::message::P2PMessage;
use crate::packet::p2p_packet::{P2PPacket, PacketContent};

pub struct CryptoReader {
    secret: String,
}

impl CryptoReader {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn process(&mut self, packet_content: &PacketContent) -> Option<P2PMessage> {
        let secret = decode_from_str(&self.secret);
        let data = decode_from_str(&packet_content.data);
        let content = aes::decode(secret.as_slice(), &data).ok()?;

        let content = String::from_utf8(content).unwrap();
        serde_json::from_str(&content).ok()
    }
}

pub struct PacketExtractor {
    temp_packet: Option<P2PPacket>,
}

impl Default for PacketExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketExtractor {
    pub fn new() -> Self {
        Self {
            temp_packet: None
        }
    }

    pub fn extract(&mut self, data: &str) -> Option<PacketContent> {
        let packets = packets_from_string(data);
        for packet in packets {
            if packet.with_head && packet.with_tail {
                let packet_content = self.gen_packet_content(&packet.content);
                return Some(packet_content);
            } else {
                if self.temp_packet.is_some() {
                    self.temp_packet = self.temp_packet.as_mut().map(|x| { x.concat(&packet) });
                } else {
                    self.temp_packet = Some(packet);
                }
                let temp = self.temp_packet.as_ref().unwrap();
                if temp.with_tail && temp.with_head {
                    let packet_content = self.gen_packet_content(&temp.content);
                    self.temp_packet = None;
                    return Some(packet_content);
                }
            }
        }
        None
    }

    fn gen_packet_content(&self, str: &str) -> PacketContent {
        let decoded = decode_from_str(str);
        let decoded = String::from_utf8(decoded).unwrap();
        decoded.as_str().into()
    }
}

fn packets_from_string(str: &str) -> Vec<P2PPacket> {
    let mut packets: Vec<P2PPacket> = vec![];
    let mut res: Option<String> = None;
    let mut packet: Option<P2PPacket> = None;
    for char in str.chars() {
        packet = packet.or_else(|| Some(Default::default()));
        res = res.or_else(|| Some(String::from("")));
        if char == '<' {
            res = Some(String::from(""));
            packet = packet.map(|mut x| {
                x.with_head = true;
                x
            });
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
            res = res.map(|mut x| {
                x.push(char);
                x
            });
        }
    }
    if packet.is_some() {
        packet = packet.map(|mut x| {
            x.content = res.unwrap();
            x
        });
        packets.push(packet.unwrap());
    }
    packets
}