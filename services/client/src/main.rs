use std::env::args;
use std::sync::Arc;
use uuid::Uuid;
use crate::config::Config;
use crate::http::HttpClient;
use crate::keystore::storage::KeyDB;
use crate::p2p::channel::create_signal_channel;
use crate::p2p::client::P2PClient;
use crate::session::SessionClient;
use crate::web_server::WebServer;

mod session;
mod route;
mod errors;
mod http;
mod p2p;
mod web_server;
mod config;
mod utils;
mod keystore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = args().nth(1);
    let config = match env {
        None => Default::default(),
        Some(config_path) => Config::new(&config_path).await.unwrap(),
    };

    let server_config = config.web_server;
    let p2p_config = config.p2p;

    let (tx, rx) = create_signal_channel();
    let key_db = KeyDB::init().await.unwrap();
    let key_db = Arc::new(key_db);
    let session_client = SessionClient::new(key_db.clone());
    let http_client = HttpClient::new(&server_config.server_host);
    let device_id = generate_device_id(&p2p_config.client_name, &session_client).await;

    let p2p_client = P2PClient::new(
        p2p_config,
        tx.clone(),
        rx,
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

async fn generate_device_id(client_name: &str, session_client: &SessionClient) -> String {
    let mut device_id = session_client.get_device_id(client_name).await;
    if device_id.is_none() {
        let temp = Uuid::new_v4().to_string();
        session_client.set_device_id(client_name, &temp).await;
        device_id = Some(temp);
    }
    device_id.unwrap()
}