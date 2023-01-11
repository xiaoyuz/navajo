pub fn encode_to_str(data: &[u8]) -> String {
    base64::encode(data)
}

pub fn decode_from_str(data: &str) -> Vec<u8> {
    base64::decode(data).unwrap()
}