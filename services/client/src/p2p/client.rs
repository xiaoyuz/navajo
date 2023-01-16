use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use tokio::{io, spawn};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};
use common::errors::{NavajoError, NavajoResult};
use common::errors::NavajoErrorRepr::SocketError;
use p2p::message::Message::PingMessage;
use p2p::message::P2PMessage;
use p2p::packet::readers::{CryptoReader, PacketExtractor};
use p2p::packet::writers::{MessageWriter, Writer};
use crate::p2p::channel::{ChannelSignal, create_thread_close_channel};
use crate::p2p::channel::ChannelSignal::{Message, RecycleChannelThread, SocketClosed};
use crate::session::SessionClient;

type ChannelSignalSender = Arc<mpsc::Sender<ChannelSignal>>;
type ChannelSignalReceiver = Arc<Mutex<mpsc::Receiver<ChannelSignal>>>;

#[derive(Clone, Deserialize)]
pub struct P2PConfig {
    pub local_port: String,
    pub server_port: String,
    pub server_host: String,
}

#[derive(Clone)]
pub struct P2PClient {
    connected: bool,
    config: P2PConfig,
    channel_tx: ChannelSignalSender,
    channel_rx: ChannelSignalReceiver,
    session_client: Arc<SessionClient>,
    device_id: String,
    message_writer: Arc<MessageWriter>,
}

impl P2PClient {
    pub fn new(
        config: P2PConfig,
        channel_tx: ChannelSignalSender,
        channel_rx: ChannelSignalReceiver,
        session_client: Arc<SessionClient>,
        device_id: String,
    ) -> Self {
        Self {
            connected: false,
            config,
            channel_tx,
            channel_rx,
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
        if self.connected {
            return Ok(());
        }
        let socket = TcpSocket::new_v4()?;

        let server_url = format!("{}:{}", self.config.server_host, self.config.server_port);
        let addr = server_url.parse().unwrap();
        let stream = socket.connect(addr).await?;
        let (r, w) = io::split(stream);
        self.connected = true;
        println!("Server Connected");

        let channel_rx = self.channel_rx.clone();
        let (close_tx, mut close_rx) = create_thread_close_channel();

        self.start_socket_read_thread(r, close_tx);
        self.start_channel_handler_thread(w, channel_rx);

        let ping_stop_mutex = self.start_ping_thread();

        while let Some(signal) = close_rx.recv().await {
            if let SocketClosed = signal {
                self.connected = false;
                let mut ping_stop_lock = ping_stop_mutex.lock().await;
                *ping_stop_lock = true;
            }
        }
        // Send a message to stop channel_handler thread manually
        self.channel_tx.send(RecycleChannelThread).await.unwrap();
        Err(NavajoError::new(SocketError { message: "Connection closed" }))
    }

    pub async fn send(&self, p2p_message: P2PMessage) {
        let sender = self.channel_tx.clone();
        sender.send(Message(p2p_message)).await.unwrap();
    }

    fn start_socket_read_thread(&self, r: ReadHalf<TcpStream>, close_tx: mpsc::Sender<ChannelSignal>) {
        // Socket read handler thread, to handle message sent by server
        let session_client = self.session_client.clone();
        let tcp_port = self.config.local_port.to_string();
        spawn(async move {
            socket_read_handle(r, close_tx, &session_client, tcp_port).await;
        });
    }

    fn start_channel_handler_thread(&self, w: WriteHalf<TcpStream>, channel_rx: ChannelSignalReceiver) {
        // Channel handler thread, to handler action of send message to socket
        let session_client = self.session_client.clone();
        let tcp_port = self.config.local_port.clone();
        let message_writer = self.message_writer.clone();
        spawn(async move {
            channel_handle(w, channel_rx, &session_client, tcp_port, &message_writer).await;
        });
    }

    fn start_ping_thread(&self) -> Arc<Mutex<bool>> {
        // Ping recycle thread
        let ping_session_client = self.session_client.clone();
        let ping_device_id = self.device_id.clone();
        let ping_channel_tx = self.channel_tx.clone();
        let ping_stop_mutex = Arc::new(Mutex::new(false));
        let ping_thread_mutex = ping_stop_mutex.clone();
        spawn(async move {
            ping(&ping_session_client, ping_channel_tx, &ping_device_id, &ping_thread_mutex).await;
        });
        return ping_stop_mutex;
    }
}

async fn ping(
    session_client: &SessionClient,
    channel_tx: ChannelSignalSender,
    device_id: &str,
    ping_stop_mutex: &Mutex<bool>
) {
    loop {
        if *ping_stop_mutex.lock().await {
            return;
        }
        sleep(Duration::from_secs(5)).await;
        println!("=====ping start");
        let opt = session_client.get_device_account(device_id).await;
        if let None = opt {
            continue;
        }
        let account = opt.unwrap();
        let ping_message = PingMessage {
            address: account.address,
            device_id: device_id.to_string(),
        };
        let p2p_message: P2PMessage = (&ping_message).into();
        channel_tx.send(Message(p2p_message)).await.unwrap();
    }
}

async fn socket_read_handle(
    mut r: ReadHalf<TcpStream>,
    close_tx: mpsc::Sender<ChannelSignal>,
    session_client: &SessionClient,
    tcp_port: String,
) {
    let mut extractor = PacketExtractor::new();
    let mut buf = vec![0; 256];
    loop {
        match r.read(&mut buf).await {
            Ok(0) => {
                close_tx.send(SocketClosed).await.unwrap();
                println!("Socket closed by server");
                return ();
            },
            Ok(n) => {
                let message = handle_message(n, &buf, &mut extractor, &session_client, &tcp_port).await;
                if let Some(mes) = message {
                    println!("{:?}", mes);
                }
            },
            Err(_) => {
                close_tx.send(SocketClosed).await.unwrap();
                println!("Socket exception");
                return ();
            },
        }
    };
}

async fn channel_handle(
    mut w: WriteHalf<TcpStream>,
    channel_rx: ChannelSignalReceiver,
    session_client: &SessionClient,
    tcp_port: String,
    message_writer: &MessageWriter,
) {
    while let Some(signal) = channel_rx.lock().await.recv().await {
        match signal {
            Message(message) => {
                let encoded = encode_message(
                    session_client,
                    tcp_port.clone(),
                    message_writer,
                    message
                ).await;
                if let Some(buf) = encoded {
                    w.write_all(buf.as_slice()).await.unwrap();
                    println!("Message sent");
                }
            }
            RecycleChannelThread => return,
            _ => (),
        }
    }
}

async fn encode_message(
    session_client: &SessionClient,
    tcp_port: String,
    message_writer: &MessageWriter,
    message: P2PMessage
) -> Option<Vec<u8>> {
    let message_str: String = (&message).into();
    let session = session_client.get_session(&tcp_port).await?;
    let secret = session_client.get_secret(&tcp_port).await?;
    let params = &[session.as_str(), secret.as_str()];
    let result = message_writer.process(&message_str, params)?;
    Some(result.as_bytes().to_vec())
}

async fn handle_message(
    n: usize,
    buf: &Vec<u8>,
    extractor: &mut PacketExtractor,
    session_client: &SessionClient,
    tcp_port: &str,
) -> Option<P2PMessage> {
    let str = String::from_utf8_lossy(&buf[..n]).to_string();
    let packet_content = extractor.extract(&str)?;
    let secret = session_client.get_secret(tcp_port).await?;
    let mut crypto_reader = CryptoReader::new(&secret);
    crypto_reader.process(&packet_content)
}