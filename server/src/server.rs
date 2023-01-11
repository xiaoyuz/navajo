use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use uuid::Uuid;
use common::beans::{DeviceInfoRequest, DeviceInfoResponse};
use common::errors::{NavajoError, NavajoResult, VERIFY_SIGN_ERROR};
use ncrypto::algo::base64::encode_to_str;
use ncrypto::algo::diffie_hellman::DiffieHellman;
use crate::db::models::User;
use crate::db::repository::UserRepository;
use crate::route::device_scope_cfg;
use crate::session::SessionClient;

#[derive(Clone)]
pub struct Server {
    pub(crate) config: ServerConfig,
    pub(crate) session_client: Arc<SessionClient>,
    pub(crate) user_repository: Arc<UserRepository>,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
}

impl Server {
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

    pub async fn create_session(&self, info: &DeviceInfoRequest) -> NavajoResult<DeviceInfoResponse> {
        if !info.verify_content() {
            Err(NavajoError::new(VERIFY_SIGN_ERROR))
        } else {
            self.logic_create_session(info).await
        }
    }

    async fn logic_create_session(&self, info: &DeviceInfoRequest) -> NavajoResult<DeviceInfoResponse> {
        let dh = DiffieHellman::new();
        let client_dh_pub = &info.dh_pub;
        let server_dh_pub = &dh.public_key_to_str();
        let secret = dh.compute_shared_secret_from_str(client_dh_pub);
        let secret = encode_to_str(&secret);
        let session = Uuid::new_v4().to_string();
        let repo = self.user_repository.clone();

        let user = User {
            id: 0,
            address: info.address.to_string(),
            device_id: info.device_id.to_string(),
            session: session.clone(),
            secret
        };
        repo.insert_or_update(&user).await?;
        Ok(DeviceInfoResponse {
            session,
            dh_pub: server_dh_pub.to_string()
        })
    }
}

