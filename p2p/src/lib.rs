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
    use crate::packet::readers::{BasicReader, Reader};
    use crate::packet::writers::{MessageWriter, Writer};
    

    #[test]
    fn test_writer_reader() {
        let query = |_| {
            Some(String::from("12345678901234567890123456789012"))
        };
        let message = P2PMessage {
            message_type: 0,
            data: String::from("12345676")
        };
        let message = serde_json::to_string(&message).unwrap();
        let query = Box::new(query);
        let mut reader = BasicReader::new(query);
        let writer = MessageWriter;
        let res = writer.process(&message, &["123", "12345678901234567890123456789012"]).unwrap();
        println!("{}", res);

        let message = reader.process(&res).unwrap();
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
