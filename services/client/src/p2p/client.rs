use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use tokio::{io, select, spawn};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::sleep;
use common::errors::{NavajoError, NavajoResult};
use common::errors::NavajoErrorRepr::SocketError;
use p2p::message::Message::PingMessage;
use p2p::message::P2PMessage;
use p2p::packet::readers::{CryptoReader, PacketExtractor};
use p2p::packet::writers::{MessageWriter, Writer};
use crate::p2p::channel::create_client_channel;
use crate::session::SessionClient;

type ChannelSignalSender = Arc<mpsc::Sender<P2PMessage>>;
type ChannelSignalReceiver = mpsc::Receiver<P2PMessage>;

#[derive(Clone, Deserialize)]
pub struct P2PConfig {
    pub local_port: String,
    pub server_port: String,
    pub server_host: String,
    pub client_name: String,
}

pub struct P2PClient {
    config: P2PConfig,
    signal_channel_tx: ChannelSignalSender,
    signal_channel_rx: ChannelSignalReceiver,
    session_client: Arc<SessionClient>,
    device_id: String,
    message_writer: Arc<MessageWriter>,
}

impl P2PClient {
    pub fn new(
        config: P2PConfig,
        signal_channel_tx: ChannelSignalSender,
        signal_channel_rx: ChannelSignalReceiver,
        session_client: Arc<SessionClient>,
        device_id: String,
    ) -> Self {
        Self {
            config,
            signal_channel_tx,
            signal_channel_rx,
            session_client,
            device_id,
            message_writer: Arc::new(MessageWriter),
        }
    }

    pub async fn start(mut self) {
        spawn(async move {
            loop {
                match self.connect().await {
                    Ok(_) => println!("Already Connected"),
                    Err(_) => println!("Connection closed"),
                }
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn connect(&mut self) -> NavajoResult<()> {
        let socket = TcpSocket::new_v4()?;

        let server_url = format!("{}:{}", self.config.server_host, self.config.server_port);
        let addr = server_url.parse().unwrap();
        let stream = socket.connect(addr).await?;
        let (r, w) = io::split(stream);
        println!("Server Connected");

        let (socket_close_tx, mut socket_close_rx) = broadcast::channel(1);

        let (channel_tx, channel_rx) = create_client_channel();
        let ping_channel_tx = channel_tx.clone();

        let socket_close_write_rx = socket_close_tx.subscribe();
        let socket_close_ping_rx = socket_close_tx.subscribe();

        self.start_socket_read_thread(r, socket_close_tx);
        self.start_socket_write_thread(w, channel_rx, socket_close_write_rx);

        self.start_ping_thread(ping_channel_tx, socket_close_ping_rx);

        loop {
            select! {
                Some(signal) = self.signal_channel_rx.recv() => {
                    channel_tx.send(signal).await;
                }
                _ = socket_close_rx.recv() => {
                    break;
                }
            }
        }
        Err(NavajoError::new(SocketError { message: "Connection closed" }))
    }

    fn start_socket_read_thread(
        &self,
        r: ReadHalf<TcpStream>,
        socket_close_tx: broadcast::Sender<()>
    ) {
        // Socket read handler thread, to handle message sent by server
        let session_client = self.session_client.clone();
        let client_name = self.config.client_name.to_string();
        spawn(async move {
            socket_read_handle(r, &session_client, client_name, socket_close_tx).await;
        });
    }

    fn start_socket_write_thread(
        &self,
        w: WriteHalf<TcpStream>,
        channel_rx: mpsc::Receiver<P2PMessage>,
        socket_close_write_rx: broadcast::Receiver<()>
    ) {
        // Channel handler thread, to handler action of send message to socket
        let session_client = self.session_client.clone();
        let client_name = self.config.client_name.clone();
        let message_writer = self.message_writer.clone();
        spawn(async move {
            channel_handle(w, channel_rx, &session_client, client_name, &message_writer, socket_close_write_rx).await;
        });
    }

    fn start_ping_thread(
        &self,
        ping_channel_tx: Arc<mpsc::Sender<P2PMessage>>,
        mut socket_close_ping_rx: broadcast::Receiver<()>
    ) {
        // Ping recycle thread
        let ping_session_client = self.session_client.clone();
        let ping_device_id = self.device_id.clone();
        spawn(async move {
            select! {
                _ = socket_close_ping_rx.recv() => {
                    println!("Ping stopped");
                }
                _ = ping(&ping_session_client, ping_channel_tx, &ping_device_id) => {
                    println!("Ping over");
                }
            }
        });
    }
}

async fn ping(
    session_client: &SessionClient,
    channel_tx: ChannelSignalSender,
    device_id: &str
) {
    loop {
        sleep(Duration::from_secs(5)).await;
        println!("=====ping start");
        let opt = session_client.get_device_account(device_id).await;
        if opt.is_none() {
            continue;
        }
        let account = opt.unwrap();
        let ping_message = PingMessage {
            address: account.address,
            device_id: device_id.to_string(),
        };
        let p2p_message: P2PMessage = (&ping_message).into();
        channel_tx.send(p2p_message).await.unwrap();
    }
}

async fn socket_read_handle(
    mut r: ReadHalf<TcpStream>,
    session_client: &SessionClient,
    client_name: String,
    socket_close_tx: broadcast::Sender<()>
) {
    let mut extractor = PacketExtractor::new();
    let mut buf = vec![0; 1024];
    loop {
        match r.read(&mut buf).await {
            Ok(0) => {
                socket_close_tx.send(()).unwrap();
                println!("Socket closed by server");
                return ;
            },
            Ok(n) => {
                let message = handle_message(n, &buf, &mut extractor, session_client, &client_name).await;
                if let Some(mes) = message {
                    println!("{:?}", mes);
                }
            },
            Err(_) => {
                socket_close_tx.send(()).unwrap();
                println!("Socket exception");
                return ;
            },
        }
    };
}

async fn channel_handle(
    mut w: WriteHalf<TcpStream>,
    mut channel_rx: mpsc::Receiver<P2PMessage>,
    session_client: &SessionClient,
    client_name: String,
    message_writer: &MessageWriter,
    mut socket_close_write_rx: broadcast::Receiver<()>
) {
    loop {
        select! {
            Some(signal) = channel_rx.recv() => {
                let encoded = encode_message(
                    session_client,
                    client_name.clone(),
                    message_writer,
                    signal
                ).await;
                if let Some(buf) = encoded {
                    w.write_all(buf.as_slice()).await.unwrap();
                    println!("Message sent");
                }
            }
            _ = socket_close_write_rx.recv() => {
                break;
            }
        }
    }
}

async fn encode_message(
    session_client: &SessionClient,
    client_name: String,
    message_writer: &MessageWriter,
    message: P2PMessage
) -> Option<Vec<u8>> {
    let message_str: String = (&message).into();
    let device_id = session_client.get_device_id(&client_name).await?;
    let session = session_client.get_session(&device_id).await?;
    let secret = session_client.get_secret(&device_id).await?;
    let params = &[session.as_str(), secret.as_str()];
    let result = message_writer.process(&message_str, params)?;
    Some(result.as_bytes().to_vec())
}

async fn handle_message(
    n: usize,
    buf: &[u8],
    extractor: &mut PacketExtractor,
    session_client: &SessionClient,
    client_name: &str,
) -> Option<P2PMessage> {
    let str = String::from_utf8_lossy(&buf[..n]).to_string();
    let packet_content = extractor.extract(&str)?;
    let device_id = session_client.get_device_id(client_name).await?;
    let secret = session_client.get_secret(&device_id).await?;
    let mut crypto_reader = CryptoReader::new(&secret);
    crypto_reader.process(&packet_content)
}