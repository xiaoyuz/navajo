use std::sync::Arc;
use mysql_async::Pool;

pub mod models;
pub mod repository;

pub struct MysqlConfig {
    user: String,
    password: String,
    database: String,
    host: String,
    port: u16,
}

impl MysqlConfig {
    pub fn new(user: &str, password: &str, database: &str, host: &str, port: u16) -> Self {
        Self {
            user: user.to_string(),
            password: password.to_string(),
            database: database.to_string(),
            host: host.to_string(),
            port
        }
    }
}

impl From<&MysqlConfig> for String {
    fn from(config: &MysqlConfig) -> Self {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            config.user,
            config.password,
            config.host,
            config.port,
            config.database
        ).to_string()
    }
}

pub fn connect(config: &MysqlConfig) -> Arc<Pool> {
    let url: String = config.into();
    let pool = Pool::new(url.as_str());
    Arc::new(pool)
}

#[cfg(test)]
mod tests {
    use crate::db::{connect, MysqlConfig};
    use crate::db::models::User;
    use crate::db::repository::UserRepository;

    #[actix_rt::test]
    async fn test_user() {
        let config = MysqlConfig::new("navajo", "example", "navajo", "127.0.0.1", 3306);
        let pool = connect(&config);
        let repo = UserRepository::new(pool.clone());
        let user = User {
            id: 0,
            address: "123".to_string(),
            device_id: "123345".to_string(),
            session: "2111".to_string(),
            secret: "bbbbbbb".to_string()
        };
        let result = repo.insert_or_update(&user).await;
        println!("{:?}", result.is_ok());

        let user = repo.find_by_address("123").await.unwrap();
        println!("{:?}", user);
    }
}
