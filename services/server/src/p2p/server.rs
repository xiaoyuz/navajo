use std::collections::HashMap;
use std::sync::{Arc};
use tokio::net::{TcpListener};
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use common::errors::NavajoResult;
use p2p::message::Message::{ChatInfoMessage, PingMessage};
use crate::db::repository::UserRepository;
use crate::p2p::channel::{ChannelSignal, create_server_channel};
use crate::p2p::channel::ChannelSignal::{ConnectionClose, ConnectionError, RemoteMessage};
use crate::p2p::connection::Connection;
use crate::queue::QueueManager;
use crate::session::SessionClient;

type ConnectionMap = Arc<Mutex<HashMap<String, Connection>>>;
type AddressIpMap = Arc<Mutex<HashMap<String, String>>>;

#[derive(Clone)]
pub struct P2PConfig {
    pub tcp_port: String,
}

pub struct P2PServer {
    config: P2PConfig,
    connection_map: ConnectionMap,
    address_ip_map: AddressIpMap,
    session_client: Arc<SessionClient>,
    user_repository: Arc<UserRepository>,
    queue_manager: Arc<QueueManager>,
}

impl P2PServer {
    pub fn new(
        config: P2PConfig,
        session_client: Arc<SessionClient>,
        user_repository: Arc<UserRepository>,
        queue_manager: Arc<QueueManager>,
    ) -> Self {
        Self {
            config,
            connection_map: Arc::new(Default::default()),
            address_ip_map: Arc::new(Default::default()),
            session_client,
            user_repository,
            queue_manager,
        }
    }

    pub async fn start(&self) -> NavajoResult<()> {
        let (tx, rx) = create_server_channel();

        let server_url = format!("{}:{}", "127.0.0.1", self.config.tcp_port);
        let listener = TcpListener::bind(server_url).await?;

        self.start_con_dispatch_thread(listener, tx);
        self.start_channel_handle_thread(rx);
        Ok(())
    }

    fn start_con_dispatch_thread(&self, listener: TcpListener, tx: Sender<ChannelSignal>) {
        let con_map = self.connection_map.clone();
        let user_repository = self.user_repository.clone();
        spawn(async move {
            connection_dispatch(listener, tx, con_map, user_repository).await;
        });
    }

    fn start_channel_handle_thread(&self, rx: Receiver<ChannelSignal>) {
        let con_map = self.connection_map.clone();
        let addr_map = self.address_ip_map.clone();
        let queue_manager = self.queue_manager.clone();
        spawn(async move {
            channel_handle(rx, con_map, addr_map, queue_manager).await;
        });
    }
}

async fn connection_dispatch(
    listener: TcpListener,
    tx: Sender<ChannelSignal>,
    con_map: ConnectionMap,
    user_repository: Arc<UserRepository>,
) {
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let peer_addr = format!("{}", addr);
        println!("New connection, {:?}", peer_addr);

        let connection = Connection::new(user_repository.clone());
        connection.start(tx.clone(), socket).await;
        con_map.lock().await.insert(peer_addr, connection);
    }
}

async fn channel_handle(
    mut rx: Receiver<ChannelSignal>,
    con_map: ConnectionMap,
    addr_map: AddressIpMap,
    queue_manager: Arc<QueueManager>,
) {
    while let Some(command) = rx.recv().await {
        match command {
            ConnectionClose(peer_addr) => {
                println!("Close socket {:?}", peer_addr);
                con_map.lock().await.remove(&peer_addr);
            },
            ConnectionError(peer_addr) => {
                con_map.lock().await.remove(&peer_addr);
            },
            RemoteMessage { peer_addr, message } => {
                println!("Got content {:?} {:?}", &peer_addr, &message);
                match message {
                    PingMessage { address, .. } => {
                        let queue_mes = queue_manager.acquire_queue(&address).await;
                        if let Some(queue_mes) = queue_mes {
                            for mes in queue_mes {
                                if let Some(con) = con_map.lock().await.get(&peer_addr) {
                                    con.call(&address, (&mes).into()).await;
                                }
                            }
                            queue_manager.remove(&address).await;
                        }
                        addr_map.lock().await.insert(address, peer_addr);
                    },
                    ChatInfoMessage { ref to_address, .. } => {
                        let addr_map = addr_map.lock().await;
                        let ip = addr_map.get(to_address);
                        if let None = ip {
                            continue;
                        }
                        if let Some(con) = con_map.lock().await.get(ip.unwrap()) {
                            con.call(to_address, (&message).into()).await;
                        } else {
                            queue_manager.add_queue(&message).await;
                        }
                    }
                }
            }
        }
    }
}