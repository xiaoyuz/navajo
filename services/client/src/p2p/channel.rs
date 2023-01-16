use std::sync::Arc;
use tokio::sync::mpsc;
use p2p::message::P2PMessage;

#[derive(Debug, Clone)]
pub enum ChannelSignal {
    SocketClosed,
    Message(P2PMessage),
    RecycleChannelThread,
}

pub fn create_thread_close_channel() -> (mpsc::Sender<ChannelSignal>, mpsc::Receiver<ChannelSignal>) {
    mpsc::channel(32)
}

pub fn create_client_channel() -> (Arc<mpsc::Sender<ChannelSignal>>, mpsc::Receiver<ChannelSignal>) {
    let (tx, rx) = mpsc::channel(1024);
    (Arc::new(tx), rx)
}