use std::sync::Arc;
use tokio::sync::mpsc;
use p2p::message::P2PMessage;

pub fn create_client_channel() -> (Arc<mpsc::Sender<P2PMessage>>, mpsc::Receiver<P2PMessage>) {
    let (tx, rx) = mpsc::channel(1024);
    (Arc::new(tx), rx)
}

pub fn create_signal_channel() -> (Arc<mpsc::Sender<P2PMessage>>, mpsc::Receiver<P2PMessage>) {
    let (tx, rx) = mpsc::channel(1024);
    (Arc::new(tx), rx)
}