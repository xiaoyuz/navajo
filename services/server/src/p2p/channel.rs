use tokio::sync::mpsc::{channel, Receiver, Sender};
use p2p::message::Message;

#[derive(Debug)]
pub enum ChannelSignal {
    ConnectionClose(String),
    ConnectionError(String),
    RemoteMessage { peer_addr: String, message: Message },
}

pub fn create_connection_channel() -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
    channel(1024)
}

pub fn create_server_channel() -> (Sender<ChannelSignal>, Receiver<ChannelSignal>) {
    channel(1024)
}