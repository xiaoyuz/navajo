use std::error::Error;
use std::sync::Arc;
use common::beans::{ApiResponse, DeviceInfoRequest, DeviceInfoResponse};

pub struct HttpClient {
    host: String,
}

impl HttpClient {
    pub fn new(host: &str) -> Arc<Self> {
        Arc::new(Self { host: String::from(host) })
    }

    pub async fn create_session(&self, body: &DeviceInfoRequest) -> Result<DeviceInfoResponse, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/device/create_session", self.host);
        let resp = client.post(url).json(&body).send().await?;
        let response: ApiResponse<DeviceInfoResponse> = resp.json().await?;
        Ok(response.content)
    }
}