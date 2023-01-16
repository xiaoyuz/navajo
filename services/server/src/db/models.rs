use mysql_async::prelude::FromRow;
use mysql_async::{from_row, FromRowError, Row};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct User {
    pub id: i32,
    pub address: String,
    pub device_id: String,
    pub session: String,
    pub secret: String,
}

impl FromRow for User {
    fn from_row(row: Row) -> Self where Self: Sized {
        Self {
            id: row.get(0).unwrap(),
            address: row.get(1).unwrap(),
            device_id: row.get(2).unwrap(),
            session: row.get(3).unwrap(),
            secret: row.get(4).unwrap()
        }
    }

    fn from_row_opt(row: Row) -> Result<Self, FromRowError> where Self: Sized {
        let user = from_row(row);
        Ok(user)
    }
}