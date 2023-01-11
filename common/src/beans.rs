use serde::{Deserialize, Serialize};
use crate::key_pair::verify;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub message: String,
    pub content: T,
}

impl<T> ApiResponse<T> {

    pub fn empty_success() -> ApiResponse<()> {
        ApiResponse {
            code: 0,
            message: String::from(""),
            content: (),
        }
    }

    pub fn success(content: T) -> ApiResponse<T> {
        ApiResponse {
            code: 0,
            message: String::from(""),
            content,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeviceInfoRequest {
    pub device_id: String,
    pub content: String,
    pub public_key: String,
    pub address: String,
    pub sign: String,
    pub dh_pub: String,
}

impl DeviceInfoRequest {
    pub fn verify_content(&self) -> bool {
        verify(&self.content, &self.sign, &self.public_key)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeviceInfoResponse {
    pub session: String,
    pub dh_pub: String,
}