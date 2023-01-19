use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand_core::OsRng;

pub fn gen_key_pair() -> Vec<u8> {
    let mut csprng = OsRng{};
    Keypair::generate(&mut csprng).to_bytes().to_vec()
}

pub fn gen_public_key(keypair: &[u8]) -> Vec<u8> {
    let keypair = Keypair::from_bytes(keypair).unwrap();
    keypair.public.as_bytes().to_vec()
}

pub fn sign(keypair: &[u8], data: &str) -> Vec<u8> {
    let keypair = Keypair::from_bytes(keypair).unwrap();
    keypair.sign(data.as_bytes()).to_bytes().to_vec()
}

pub fn verify(src: &[u8], sign: &[u8], public_key: &[u8]) -> bool {
    let public_key = PublicKey::from_bytes(public_key).unwrap();
    let sign = Signature::from_bytes(sign).unwrap();
    let res = public_key.verify(src, &sign);
    res.is_ok()
}