pub mod storage;

#[cfg(test)]
mod tests {
    use crate::keystore::storage::KeyDB;

    #[actix_rt::test]
    async fn test_keystore() {
        let key_db = KeyDB::init().await.unwrap();
        key_db.set("test7", "value7").await;
        key_db.set("test1", "value1").await;
        key_db.set("test2", "value2").await;
        let value = key_db.get("test7").await;
        println!("{:?}", value);
        key_db.remove("test7").await;
    }
}