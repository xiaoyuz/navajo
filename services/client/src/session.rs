use std::sync::Arc;
use common::account::Account;
use crate::keystore::storage::KeyDB;

const SESSION_EXPIRE_SECONDS: u64 = 30 * 24 * 60 * 60; // 30 days

const CLIENT_DEVICE_ACCOUNT: &str = "client_device_account:";

const CLIENT_SESSION: &str = "client_session:";
const CLIENT_SECRET: &str = "client_secret:";
const CLIENT_DEVICE_ID: &str = "client_device_id:";

pub struct SessionClient {
    key_db: Arc<KeyDB>,
}

impl SessionClient {
    pub fn new(key_db: Arc<KeyDB>) -> Arc<Self> {
        Arc::new(Self { key_db })
    }

    pub async fn get_device_account(&self, device_id: &str) -> Option<Account> {
        let json_str = self.key_db.get(format!("{}{}", CLIENT_DEVICE_ACCOUNT, device_id).as_str()).await?;
        Some(json_str.into())
    }

    pub async fn set_device_account(&self, device_id: &str, account: &Account) {
        let key = format!("{}{}", CLIENT_DEVICE_ACCOUNT, device_id);
        let json_str: String = account.into();
        self.key_db.set(&key, &json_str).await;
    }

    pub async fn del_device_account(&self, device_id: &str) {
        self.key_db.remove(format!("{}{}", CLIENT_DEVICE_ACCOUNT, device_id).as_str()).await;
    }

    pub async fn get_session(&self, device_id: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_SESSION, device_id);
        self.key_db.get(&key).await
    }

    pub async fn set_session(&self, device_id: &str, session: &str) {
        let key = format!("{}{}", CLIENT_SESSION, device_id);
        self.key_db.set(&key, session).await;
    }

    pub async fn del_session(&self, device_id: &str) {
        self.key_db.remove(format!("{}{}", CLIENT_SESSION, device_id).as_str()).await;
    }

    pub async fn get_secret(&self, device_id: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_SECRET, device_id);
        self.key_db.get(&key).await
    }

    pub async fn set_secret(&self, device_id: &str, secret: &str) {
        let key = format!("{}{}", CLIENT_SECRET, device_id);
        self.key_db.set(&key, secret).await;
    }

    pub async fn del_secret(&self, device_id: &str) {
        self.key_db.remove(format!("{}{}", CLIENT_SECRET, device_id).as_str()).await;
    }

    pub async fn get_device_id(&self, client_name: &str) -> Option<String> {
        let key = format!("{}{}", CLIENT_DEVICE_ID, client_name);
        self.key_db.get(&key).await
    }

    pub async fn set_device_id(&self, client_name: &str, device_id: &str) {
        let key = format!("{}{}", CLIENT_DEVICE_ID, client_name);
        self.key_db.set(&key, device_id).await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use common::account::Account;
    use crate::keystore::storage::KeyDB;
    use crate::session::SessionClient;

    #[actix_rt::test]
    async fn test_session() {
        let key_db = KeyDB::init().await.unwrap();
        let rc = Arc::new(key_db);
        let session_client1 = SessionClient::new(rc.clone());

        let account = Account::new();
        session_client1.set_device_account("1111111", &account).await;
        let res = session_client1.get_device_account("1111111").await;
        println!("{:?}", res);
    }
}