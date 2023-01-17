use nredis::RedisClient;
use crate::config::Config;
use crate::db::connect;
use crate::db::repository::UserRepository;
use crate::p2p::server::P2PServer;
use crate::queue::QueueManager;
use crate::server::Server;
use crate::session::SessionClient;

mod session;
mod db;
mod queue;
mod p2p;
mod server;
mod errors;
pub mod route;
mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new().unwrap();

    let mysql_pool = connect(&config.mysql);
    let user_repository = UserRepository::new(mysql_pool.clone());
    let redis_client = RedisClient::new(Box::new(config.redis));
    let session_client = SessionClient::new(redis_client.clone());

    let server = Server {
        config: config.server,
        session_client: session_client.clone(),
        user_repository: user_repository.clone(),
    };

    let queue_manager = QueueManager::new(redis_client.clone());
    let p2p_server = P2PServer::new(
        config.p2p,
        session_client,
        user_repository,
        queue_manager
    );
    p2p_server.start().await.unwrap();

    server.start().await
}
