use mac_address::get_mac_address;
use uuid::Uuid;
use common::errors::{MAC_ADDR_ERROR, NavajoError, NavajoResult};
use ncrypto::algo::{base64, sha256};
use crate::generate_device_id;

pub fn gen_device_id() -> NavajoResult<String> {
    let mac_addr = get_mac_address().map_err(|_| NavajoError::new(MAC_ADDR_ERROR))?;
    let device_id = match mac_addr {
        Some(addr) => {
            let bs = sha256::encode(&addr.bytes());
            base64::encode_to_str(bs.as_slice())
        }
        None => Uuid::new_v4().to_string()
    };
    Ok(device_id)
}