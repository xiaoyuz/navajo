use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::p2p::channel::ChannelSignal::{ConnectionClose, ConnectionError, RemoteMessage};
use crate::p2p::channel::{ChannelSignal, create_connection_channel};

type ConnectionReceiver = Arc<Mutex<Receiver<String>>>;

pub struct Connection {
    con_tx: Sender<String>,
    con_rx: ConnectionReceiver,
}

impl Connection {
    pub fn new() -> Self {
        let (con_tx, con_rx) = create_connection_channel();
        Self { con_tx, con_rx: Arc::new(Mutex::new(con_rx)) }
    }

    pub async fn start(&self, server_channel_tx: Sender<ChannelSignal>, socket: TcpStream) {
        let peer_addr = format!("{}", socket.peer_addr().unwrap());
        let socket = Arc::new(Mutex::new(socket));
        let socket_read = socket.clone();

        self.start_channel_handle_thread(socket);
        self.start_socket_read_thread(socket_read, server_channel_tx, peer_addr);
    }

    // TODO: fix me
    pub async fn call(&self, message: String) {
        self.con_tx.send(message).await.unwrap();
    }

    fn start_channel_handle_thread(&self, socket: Arc<Mutex<TcpStream>>) {
        let con_rx = self.con_rx.clone();
        // Call from others
        spawn(async move {
            while let Some(message) = con_rx.lock().await.recv().await {
                socket.lock().await.write_all(message.as_bytes()).await.unwrap();
            }
        });
    }

    fn start_socket_read_thread(
        &self,
        socket: Arc<Mutex<TcpStream>>,
        server_channel_tx: Sender<ChannelSignal>,
        peer_addr: String
    ) {
        // Serve the socket read
        spawn(async move {
            let mut buf = vec![0; 256];
            loop {
                match socket.lock().await.read(&mut buf).await {
                    Ok(0) => {
                        server_channel_tx.send(ConnectionClose(peer_addr.clone())).await.unwrap();
                        return;
                    }
                    Ok(n) => {
                        let str = String::from_utf8_lossy(&buf[..n]).to_string();
                        println!("{:?}", str);
                        server_channel_tx.send(RemoteMessage {
                            addr: peer_addr.clone(),
                            content: str,
                        }).await.unwrap();
                    }
                    Err(_) => {
                        server_channel_tx.send(ConnectionError(peer_addr.clone())).await.unwrap();
                        return;
                    }
                }
            }
        });
    }
}

