use std::sync::Arc;
use mysql_async::Pool;

pub mod models;
pub mod repository;

pub fn connect(url: &str) -> Arc<Pool> {
    let pool = Pool::new(url);
    Arc::new(pool)
}

#[cfg(test)]
mod tests {
    use crate::db::connect;
    use crate::db::models::User;
    use crate::db::repository::UserRepository;

    #[actix_rt::test]
    async fn test_user() {
        let pool = connect("mysql://navajo:example@127.0.0.1:3306/navajo");
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
