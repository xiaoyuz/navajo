pub mod aes;
pub mod base58;
pub mod diffie_hellman;
pub mod sha256;
pub mod base64;

#[cfg(test)]
mod tests {
    use crate::algo::aes;
    use crate::algo::base58;
    use crate::algo::base64;
    use crate::algo::base64::decode_from_str;
    use crate::algo::diffie_hellman::DiffieHellman;
    use crate::algo::sha256;

    #[test]
    fn test_aes_bytes() {
        let data = "MTM0NTY3";
        let sec = "fgVobm2TEGDyWX6GOJrXTuuUoNbfeMpJSa0WhdTcO0k=";

        let sec = decode_from_str(sec);
        let data = decode_from_str(data);
        println!("{:?}", data);

        let encrypted = aes::encode(&sec, &data).unwrap();
        println!("{:?}", encrypted);

        let b64 = base64::encode_to_str(&encrypted);
        println!("{:?}", b64);

        let decrypted = aes::decode(&sec, &encrypted).unwrap();
        println!("{:?}", decrypted);
    }

    #[test]
    fn test_base58() {
        let data = String::from("test");
        println!("{:?}", data);

        let base58 = base58::encode(data.as_bytes());
        println!("{:?}", base58);

        let data = base58::decode(&base58);
        println!("{:?}", String::from_utf8(data).unwrap());
    }

    #[test]
    fn test_dh() {
        let dh1 = DiffieHellman::new();
        let dh2 = DiffieHellman::new();

        let pub1_str = dh1.public_key_to_str();
        let pub2_str = dh2.public_key_to_str();

        println!("{:?}", pub1_str);
        println!("{:?}", pub2_str);

        let share1 = dh1.compute_shared_secret_from_str(&pub2_str);
        let share2 = dh2.compute_shared_secret_from_str(&pub1_str);

        println!("{:?}", share1);
        println!("{:?}", share2);
    }

    #[test]
    fn test_sha256() {
        let src = "hello";
        let result = sha256::encode(src.as_bytes());
        println!("{:?}", result);
    }

    #[test]
    fn test_base64() {
        let src = "hello";
        println!("{:?}", src.as_bytes());
        let result = base64::encode_to_str(src.as_bytes());
        let result = base64::decode_from_str(&result);
        println!("{:?}", result);
    }
}
