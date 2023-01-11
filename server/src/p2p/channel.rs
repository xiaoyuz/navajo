use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum ChannelSignal {
    ConnectionClose(String),
    ConnectionError(String),
    RemoteMessage { addr: String, content: String },
}

pub fn create_connection_channel() -> (Sender<String>, Receiver<String>) {
    channel(1024)
}

pub fn create_server_channel() -> (Sender<ChannelSignal>, Receiver<ChannelSignal>) {
    channel(1024)
}