use std::env;
use common::errors::NavajoResult;
use nredis::RedisConfig;
use crate::db::MysqlConfig;
use crate::p2p::server::P2PConfig;
use crate::server::ServerConfig;

const PORT: u16 = 28100;
const TCP_PORT: &str = "6000";
const REDIS_HOST: &str = "redis://127.0.0.1/";
const MYSQL_HOST: &str = "127.0.0.1";
const MYSQL_PORT: u16 = 3306;
const MYSQL_DATABASE: &str = "navajo";
const MYSQL_USER: &str = "navajo";
const MYSQL_PASSWORD: &str = "example";

pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub p2p: P2PConfig,
    pub mysql: MysqlConfig,
}

impl Config {
    pub fn new() -> NavajoResult<Self> {
        let port = env::var("NAVAJO_WEB_PORT").unwrap_or_else(|_| PORT.to_string()).parse().unwrap();
        let tcp_port = env::var("NAVAJO_TCP_PORT").unwrap_or_else(|_| TCP_PORT.to_string());
        let redis_host = env::var("NAVAJO_REDIS_HOST").unwrap_or_else(|_| REDIS_HOST.to_string());
        let mysql_host = env::var("NAVAJO_MYSQL_HOST").unwrap_or_else(|_| MYSQL_HOST.to_string());
        let mysql_port = env::var("NAVAJO_MYSQL_PORT").unwrap_or_else(|_| MYSQL_PORT.to_string()).parse().unwrap();
        let mysql_database = env::var("NAVAJO_MYSQL_DATABASE").unwrap_or_else(|_| MYSQL_DATABASE.to_string());
        let mysql_user = env::var("NAVAJO_MYSQL_USER").unwrap_or_else(|_| MYSQL_USER.to_string());
        let mysql_password = env::var("NAVAJO_MYSQL_PASSWORD").unwrap_or_else(|_| MYSQL_PASSWORD.to_string());

        let p2p = P2PConfig { tcp_port };
        let server = ServerConfig { port };
        let redis = RedisConfig {
            host: redis_host,
        };
        let mysql = MysqlConfig::new(&mysql_user, &mysql_password, &mysql_database, &mysql_host, mysql_port);
        let config = Config { server, redis, p2p, mysql };
        Ok(config)
    }
}