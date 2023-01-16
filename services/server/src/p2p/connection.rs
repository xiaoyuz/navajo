use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use p2p::message::{Message, P2PMessage};
use p2p::packet::readers::{CryptoReader, PacketExtractor};
use p2p::packet::writers::{MessageWriter, Writer};
use crate::db::repository::UserRepository;
use crate::p2p::channel::ChannelSignal::{ConnectionClose, ConnectionError, RemoteMessage};
use crate::p2p::channel::{ChannelSignal, create_connection_channel};

type ConnectionReceiver = Arc<Mutex<Receiver<Vec<u8>>>>;

pub struct Connection {
    con_tx: Sender<Vec<u8>>,
    con_rx: ConnectionReceiver,
    user_repository: Arc<UserRepository>,
    message_writer: Arc<MessageWriter>,
}

impl Connection {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        let (con_tx, con_rx) = create_connection_channel();
        Self {
            con_tx,
            con_rx: Arc::new(Mutex::new(con_rx)),
            user_repository,
            message_writer: Arc::new(MessageWriter),
        }
    }

    pub async fn start(&self, server_channel_tx: Sender<ChannelSignal>, socket: TcpStream) {
        let peer_addr = format!("{}", socket.peer_addr().unwrap());
        let socket = Arc::new(Mutex::new(socket));
        let socket_read = socket.clone();

        self.start_channel_handle_thread(socket);
        self.start_socket_read_thread(socket_read, server_channel_tx, peer_addr);
    }

    pub async fn call(&self, to_address: &str, message: P2PMessage) {
        if let Some(encoded) = encode_message(to_address, &self.user_repository, &self.message_writer, message).await {
            self.con_tx.send(encoded).await.unwrap();
        }
    }

    fn start_channel_handle_thread(&self, socket: Arc<Mutex<TcpStream>>) {
        let con_rx = self.con_rx.clone();
        // Call from others
        spawn(async move {
            while let Some(message) = con_rx.lock().await.recv().await {
                println!("Preper Forward message");
                socket.lock().await.write_all(message.as_slice()).await.unwrap();
                println!("Forward message");
            }
        });
    }

    fn start_socket_read_thread(
        &self,
        socket: Arc<Mutex<TcpStream>>,
        server_channel_tx: Sender<ChannelSignal>,
        peer_addr: String
    ) {
        let user_repository = self.user_repository.clone();
        // Serve the socket read
        spawn(async move {
            let mut extractor = PacketExtractor::new();
            let mut buf = vec![0; 256];
            loop {
                match socket.lock().await.read(&mut buf).await {
                    Ok(0) => {
                        server_channel_tx.send(ConnectionClose(peer_addr.clone())).await.unwrap();
                        return;
                    }
                    Ok(n) => {
                        handle_message(
                            n,
                            &buf,
                            &mut extractor,
                            &server_channel_tx,
                            peer_addr.clone(),
                            &user_repository.clone()
                        ).await;
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

async fn handle_message(
    n: usize,
    buf: &Vec<u8>,
    extractor: &mut PacketExtractor,
    server_channel_tx: &Sender<ChannelSignal>,
    addr: String,
    user_repository: &UserRepository,
) -> Option<P2PMessage> {
    let str = String::from_utf8_lossy(&buf[..n]).to_string();
    println!("{:?}", str);
    let packet_content = extractor.extract(&str)?;
    let session = packet_content.session.as_str();
    let users = user_repository.find_by_session(session).await?;
    let user = users.first()?;
    let secret = user.secret.as_str();
    let mut crypto_reader = CryptoReader::new(secret);
    let p2p_message = crypto_reader.process(&packet_content)?;
    let message: Message = (&p2p_message).into();
    server_channel_tx.send(RemoteMessage {
        peer_addr: addr,
        message,
    }).await.unwrap();
    Some(p2p_message)
}

async fn encode_message(
    to_address: &str,
    user_repository: &UserRepository,
    message_writer: &MessageWriter,
    message: P2PMessage,
) -> Option<Vec<u8>> {
    let message_str: String = (&message).into();
    let users = user_repository.find_by_address(to_address).await?;
    let user = users.first()?;
    let session = &user.session;
    let secret = &user.secret;
    let params = &[session.as_str(), secret.as_str()];
    let result = message_writer.process(&message_str, params)?;
    Some(result.as_bytes().to_vec())
}

