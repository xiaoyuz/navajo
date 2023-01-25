extern crate core;

pub mod account;
pub mod key_pair;
pub mod beans;
pub mod errors;

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};


    #[test]
    fn test_account() {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        println!("{}", t);
    }
}
