use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;
use common::account::Account;
use common::beans::{DeviceInfoRequest, DeviceInfoResponse};
use common::errors::{HTTP_ERROR, INVALID_DEVICE_ID, NavajoError, NavajoResult};
use ncrypto::algo::base64::encode_to_str;
use ncrypto::algo::diffie_hellman::DiffieHellman;
use p2p::message::Message::PingMessage;
use p2p::message::P2PMessage;
use crate::http::HttpClient;
use crate::p2p::channel::ChannelSignal;
use crate::p2p::channel::ChannelSignal::Message;
use crate::route::device_scope_cfg;
use crate::session::SessionClient;

#[derive(Clone)]
pub struct WebServer {
    config: WebServerConfig,
    session_client: Arc<SessionClient>,
    http_client: Arc<HttpClient>,
    device_id: String,
    p2p_client_sender: Arc<Sender<ChannelSignal>>,
}

#[derive(Clone)]
pub struct WebServerConfig {
    pub port: u16,
    pub tcp_port: String,
}

impl WebServer {
    pub fn new(
        config: WebServerConfig,
        session_client: Arc<SessionClient>,
        http_client: Arc<HttpClient>,
        device_id: String,
        p2p_client_sender: Arc<Sender<ChannelSignal>>,
    ) -> Self {
        Self {
            config,
            session_client,
            http_client,
            device_id,
            p2p_client_sender,
        }
    }

    pub async fn start(self) -> std::io::Result<()> {
        let port = self.config.port;
        let arc_state = Data::new(self);
        HttpServer::new(move || {
            App::new()
                .app_data(arc_state.clone())
                .service(web::scope("device").configure(device_scope_cfg))
        })
            .bind(("127.0.0.1", port))?
            .run()
            .await
    }

    pub async fn register(&self) -> NavajoResult<Account> {
        let session_client = self.session_client.clone();
        let device_id  = &self.device_id;

        let mut account = session_client.get_device_account(device_id).await;
        if let None = account {
            let temp = Account::new();
            session_client.set_device_account(device_id, &temp).await;
            account = Some(temp);
        }
        Ok(account.unwrap())
    }

    pub async fn create_session(&self) -> NavajoResult<(String, String)> {
        self.logic_create_session().await
    }

    async fn logic_create_session(&self) -> NavajoResult<(String, String)> {
        let session_client = self.session_client.clone();
        let http_client = self.http_client.clone();
        let device_id  = &self.device_id;

        let account = session_client.get_device_account(device_id)
            .await.ok_or(NavajoError::new(INVALID_DEVICE_ID))?;

        let dh = DiffieHellman::new();
        let dh_pub = dh.public_key_to_str();
        let content = Uuid::new_v4().to_string();
        let address = &account.address;
        let public_key = account.key_pair.gen_public_key();
        let sign = account.sign_data(&content);
        let body = DeviceInfoRequest {
            device_id: String::from(device_id),
            content,
            public_key,
            address: String::from(address),
            sign,
            dh_pub,
        };
        let DeviceInfoResponse { session, dh_pub } = http_client.create_session(&body)
            .await.map_err(|_| NavajoError::new(HTTP_ERROR))?;
        let shared_secret = dh.compute_shared_secret_from_str(&dh_pub);
        let shared_secret = encode_to_str(&shared_secret);
        session_client.set_session(&self.config.tcp_port, &session).await;
        session_client.set_secret(&self.config.tcp_port, &shared_secret).await;
        Ok((session, shared_secret))
    }

    pub async fn test_p2p(&self) -> NavajoResult<()> {
        let ping_message = PingMessage {
            address: "111".to_string(),
            device_id: "222".to_string()
        };
        let p2p_message = P2PMessage {
            message_type: 0,
            data: (&ping_message).into(),
        };
        self.p2p_client_sender.send(Message(p2p_message)).await.unwrap();
        Ok(())
    }
}