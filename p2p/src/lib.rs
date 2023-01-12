extern crate core;

pub mod packet;
pub mod message;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    
    use crate::message::P2PMessage;
    use crate::packet::p2p_packet::P2PPacket;
    use crate::packet::readers::{CryptoReader, PacketExtractor};
    use crate::packet::writers::{MessageWriter, Writer};
    

    #[test]
    fn test_writer_reader() {
        let message = P2PMessage {
            message_type: 0,
            data: String::from("12345676")
        };
        let message = serde_json::to_string(&message).unwrap();

        let writer = MessageWriter;
        let res = writer.process(&message, &["123", "fgVobm2TEGDyWX6GOJrXTuuUoNbfeMpJSa0WhdTcO0k="]).unwrap();
        println!("{}", res);

        let mut extractor = PacketExtractor::new();
        let packet_content = extractor.extract(&res).unwrap();

        let mut reader = CryptoReader::new("fgVobm2TEGDyWX6GOJrXTuuUoNbfeMpJSa0WhdTcO0k=");

        let message = reader.process(&packet_content).unwrap();
        println!("{:?}", message);
    }

    #[test]
    fn test_packet() {
        let packet1 = P2PPacket {
            content: String::from("abc"),
            with_head: true,
            with_tail: false,
        };
        let packet2 = P2PPacket {
            content: String::from("yyyy"),
            with_head: false,
            with_tail: true,
        };
        let packet = packet1.concat(&packet2);
        println!("{:?}", packet);
    }
}
