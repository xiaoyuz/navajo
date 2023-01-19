use std::sync::Arc;
use mysql_async::{Conn, params, Pool};
use mysql_async::prelude::{Query, WithParams};
use common::errors::{DB_ERROR, NavajoError, NavajoResult};
use crate::db::models::User;

pub struct UserRepository {
    pool: Arc<Pool>,
}

impl UserRepository {
    pub fn new(pool: Arc<Pool>) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    pub async fn find_by_address(&self, address: &str) -> Option<Vec<User>> {
        let mut conn = self.get_conn().await?;
        "SELECT * FROM user WHERE address = :address"
            .with(params! { address }).fetch(&mut conn)
            .await.ok()
    }

    pub async fn find_by_device_id(&self, device_id: &str) -> Option<Vec<User>> {
        let mut conn = self.get_conn().await?;
        "SELECT * FROM user WHERE device_id = :device_id"
            .with(params! { device_id }).fetch(&mut conn)
            .await.ok()
    }

    pub async fn find_by_session(&self, session: &str) -> Option<Vec<User>> {
        let mut conn = self.get_conn().await?;
        "SELECT * FROM user WHERE session = :session"
            .with(params! { session }).fetch(&mut conn)
            .await.ok()
    }

    pub async fn insert_or_update(&self, user: &User) -> NavajoResult<()> {
        let mut conn = self.get_conn().await.ok_or_else(|| NavajoError::new(DB_ERROR))?;
        let params = params! {
            "address" => &user.address,
            "device_id" => &user.device_id,
            "session" => &user.session,
            "secret" => &user.secret,
        };

        let insert_res = r"INSERT INTO user(address, device_id, session, secret) VALUES (:address, :device_id, :session, :secret)"
            .with(&params).run(&mut conn).await;
        if insert_res.is_err() {
            r"UPDATE user SET device_id = :device_id, session = :session, secret = :secret WHERE address = :address"
                .with(&params).run(&mut conn).await.map_err(|_| NavajoError::new(DB_ERROR)).map(|_| ())
        } else {
            Ok(())
        }
    }

    async fn get_conn(&self) -> Option<Conn> {
        self.pool.get_conn().await.ok()
    }
}