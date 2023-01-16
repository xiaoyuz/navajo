use nredis::{RedisClient, RedisConfig};
use crate::db::connect;
use crate::db::repository::UserRepository;
use crate::p2p::server::{P2PConfig, P2PServer};
use crate::queue::QueueManager;
use crate::server::{Server, ServerConfig};
use crate::session::SessionClient;

mod session;
mod db;
mod queue;
mod p2p;
mod server;
mod errors;
pub mod route;

const PORT: u16 = 28100;
const TCP_PORT: &str = "6000";
const REDIS_HOST: &str = "redis://127.0.0.1/";
const MYSQL_HOST: &str = "127.0.0.1";
const MYSQL_PORT: u16 = 3306;
const MYSQL_DATABASE: &str = "navajo";
const MYSQL_USER: &str = "navajo";
const MYSQL_PASSWORD: &str = "example";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let p2p_config = P2PConfig {
        tcp_port: TCP_PORT.to_string(),
    };
    let server_config = ServerConfig {
        port: PORT,
    };

    let redis_config = RedisConfig {
        host: String::from(REDIS_HOST),
    };
    let redis_client = RedisClient::new(Box::new(redis_config));
    let session_client = SessionClient::new(redis_client.clone());

    let mysql_url = format!("mysql://{}:{}@{}:{}/{}", MYSQL_USER, MYSQL_PASSWORD, MYSQL_HOST, MYSQL_PORT, MYSQL_DATABASE);
    let mysql_pool = connect(&mysql_url);
    let user_repository = UserRepository::new(mysql_pool.clone());

    let server = Server {
        config: server_config,
        session_client: session_client.clone(),
        user_repository: user_repository.clone(),
    };

    let queue_manager = QueueManager::new(redis_client.clone());
    let p2p_server = P2PServer::new(
        p2p_config,
        session_client,
        user_repository,
        queue_manager
    );
    p2p_server.start().await.unwrap();

    server.start().await
}
