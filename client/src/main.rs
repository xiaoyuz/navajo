use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use nredis::{RedisClient, RedisConfig};
use crate::http::HttpClient;
use crate::p2p::channel::create_client_channel;
use crate::p2p::client::{P2PClient, P2PConfig};
use crate::session::SessionClient;
use crate::web_server::{WebServer, WebServerConfig};

mod session;
mod route;
mod errors;
mod http;
mod p2p;
mod web_server;

const PORT: u16 = 8085;
const REDIS_HOST: &str = "redis://127.0.0.1/";
const TCP_PORT: &str = "7000";
const TCP_SERVER_HOST: &str = "127.0.0.1";
const TCP_SERVER_PORT: &str = "6000";
const SERVER_HOST: &str = "http://127.0.0.1:28100";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_config = WebServerConfig {
        port: PORT,
        tcp_port: TCP_PORT.to_string(),
    };
    let p2p_config = P2PConfig {
        local_port: TCP_PORT.to_string(),
        server_port: TCP_SERVER_HOST.to_string(),
        server_host: TCP_SERVER_PORT.to_string(),
    };
    let redis_config = RedisConfig {
        host: String::from(REDIS_HOST),
    };

    let (tx, rx) = create_client_channel();
    let redis_client = RedisClient::new(Box::new(redis_config));
    let session_client = SessionClient::new(redis_client.clone());
    let http_client = HttpClient::new(SERVER_HOST);
    let device_id = generate_device_id(TCP_PORT, &session_client).await;

    let mutex_rx = Arc::new(Mutex::new(rx));
    let p2p_client = P2PClient::new(
        p2p_config,
        tx.clone(),
        mutex_rx,
        session_client.clone(),
        device_id.clone(),
    );

    let web_server = WebServer::new(
        server_config,
        session_client,
        http_client.clone(),
        device_id.clone(),
        tx.clone(),
    );

    p2p_client.start().await;
    web_server.start().await
}

async fn generate_device_id(tcp_port: &str, session_client: &SessionClient) -> String {
    let mut device_id = session_client.get_device_id(tcp_port).await;
    if let None = device_id {
        let temp = Uuid::new_v4().to_string();
        session_client.set_device_id(tcp_port, &temp).await;
        device_id = Some(temp);
    }
    device_id.unwrap()
}