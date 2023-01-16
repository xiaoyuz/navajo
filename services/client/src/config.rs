use common::errors::NavajoResult;
use nredis::RedisConfig;
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{web_server::WebServerConfig, p2p::client::P2PConfig};

const PORT: u16 = 8085;
const REDIS_HOST: &str = "redis://127.0.0.1/";
const TCP_PORT: &str = "7000";
const TCP_SERVER_HOST: &str = "127.0.0.1";
const TCP_SERVER_PORT: &str = "6000";
const SERVER_HOST: &str = "http://127.0.0.1:28100";

#[derive(Deserialize)]
pub struct Config {
    pub web_server: WebServerConfig,
    pub redis: RedisConfig,
    pub p2p: P2PConfig,
}

impl Config {
    pub async fn new(config_path: &str) -> NavajoResult<Self> {
        let mut config_file = File::open(config_path).await?;
        let mut buf = Vec::new();
        config_file.read_to_end(&mut buf).await?;
        let config: Config = toml::from_slice(&buf).unwrap();
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        let web_server = WebServerConfig {
            port: PORT,
            tcp_port: TCP_PORT.to_string(),
            server_host: SERVER_HOST.to_string(),
        };
        let p2p = P2PConfig {
            local_port: TCP_PORT.to_string(),
            server_port: TCP_SERVER_PORT.to_string(),
            server_host: TCP_SERVER_HOST.to_string(),
        };
        let redis = RedisConfig {
            host: String::from(REDIS_HOST),
        };
        Self { web_server, redis, p2p }
    }
}