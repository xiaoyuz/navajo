pub mod account;
pub mod key_pair;
pub mod beans;
pub mod errors;

#[cfg(test)]
mod tests {
    use crate::account::Account;
    use super::*;

    #[test]
    fn test_account() {
        let account = Account::new();
        println!("{:?}", account);
    }
}
