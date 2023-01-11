use std::collections::HashMap;
use std::sync::{Arc};
use tokio::net::{TcpListener};
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use common::errors::NavajoResult;
use crate::db::repository::UserRepository;
use crate::p2p::channel::{ChannelSignal, create_server_channel};
use crate::p2p::channel::ChannelSignal::{ConnectionClose, ConnectionError, RemoteMessage};
use crate::p2p::connection::Connection;
use crate::queue::QueueManager;
use crate::session::SessionClient;

type ConnectionMap = Arc<Mutex<HashMap<String, Connection>>>;

#[derive(Clone)]
pub struct P2PConfig {
    pub tcp_port: String,
}

pub struct P2PServer {
    pub(crate) config: P2PConfig,
    pub(crate) connection_map: ConnectionMap,
    pub(crate) session_client: Arc<SessionClient>,
    pub(crate) user_repository: Arc<UserRepository>,
    pub(crate) queue_manager: Arc<QueueManager>,
}

impl P2PServer {
    pub async fn start(&self) -> NavajoResult<()> {
        let (tx, rx) = create_server_channel();

        let server_url = format!("{}:{}", "127.0.0.1", self.config.tcp_port);
        let listener = TcpListener::bind(server_url).await?;

        self.start_con_dispatch_thread(listener, tx);
        self.start_channel_handle_thread(rx);
        Ok(())
    }

    fn start_con_dispatch_thread(&self, listener: TcpListener, tx: Sender<ChannelSignal>) {
        let socket_con_map_arc = self.connection_map.clone();
        spawn(async move {
            connection_dispatch(listener, tx, socket_con_map_arc).await;
        });
    }

    fn start_channel_handle_thread(&self, rx: Receiver<ChannelSignal>) {
        let channel_con_map_arc = self.connection_map.clone();
        spawn(async move {
            channel_handle(rx, channel_con_map_arc).await;
        });
    }
}

async fn connection_dispatch(
    listener: TcpListener,
    tx: Sender<ChannelSignal>,
    con_map_arc: ConnectionMap
) {
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let peer_addr = format!("{}", addr);
        println!("New connection, {:?}", peer_addr);

        let connection = Connection::new();
        connection.start(tx.clone(), socket).await;
        con_map_arc.lock().await.insert(peer_addr, connection);
    }
}

async fn channel_handle(
    mut rx: Receiver<ChannelSignal>,
    con_map_arc: ConnectionMap
) {
    while let Some(command) = rx.recv().await {
        match command {
            ConnectionClose(addr) => {
                println!("Close socket {:?}", addr);
                con_map_arc.lock().await.remove(&addr);
            },
            ConnectionError(addr) => {
                con_map_arc.lock().await.remove(&addr);
            },
            RemoteMessage { addr, content } => {
                println!("Got content {:?} {:?}", addr, content);
            }
        }
    }
}