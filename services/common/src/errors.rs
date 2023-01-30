use std::fmt::{Display, Formatter};
use std::io;
use crate::errors::NavajoErrorRepr::{IoError, MessageError, SocketError};

#[derive(Debug)]
pub enum NavajoErrorRepr {
    IoError(io::Error),
    MessageError { code: u32, message: &'static str },
    SocketError { message: &'static str },
}

pub const INVALID_PARAM_ERROR: NavajoErrorRepr = MessageError { code: 101, message: "invalid param error" };
pub const ECDSA_ENCRYPT_ERROR: NavajoErrorRepr = MessageError { code: 104, message: "ECDSA encrypt error" };
pub const VERIFY_SIGN_ERROR: NavajoErrorRepr = MessageError { code: 108, message: "verify sign error" };
pub const VERIFY_HASH_ERROR: NavajoErrorRepr = MessageError { code: 109, message: "verify hash error" };
pub const INVALID_DH_ERROR: NavajoErrorRepr = MessageError { code: 110, message: "invalid dh key" };

pub const INVALID_KEY_PAIR: NavajoErrorRepr = MessageError { code: 301, message: "invalid key pair" };

pub const INVALID_DEVICE_ID: NavajoErrorRepr = MessageError { code: 401, message: "invalid device id" };
pub const INVALID_SESSION: NavajoErrorRepr = MessageError { code: 402, message: "invalid session" };

pub const DB_ERROR: NavajoErrorRepr = MessageError { code: 500, message: "db error" };
pub const HTTP_ERROR: NavajoErrorRepr = MessageError { code: 600, message: "http error" };

pub const MAC_ADDR_ERROR: NavajoErrorRepr = MessageError { code: 700, message: "mac address parse error" };
pub const LOGIN_ERROR: NavajoErrorRepr = MessageError { code: 701, message: "login error" };

pub type NavajoResult<T> = Result<T, NavajoError>;

#[derive(Debug)]
pub struct NavajoError {
    repr: NavajoErrorRepr,
}

impl From<io::Error> for NavajoError {
    fn from(err: io::Error) -> Self {
        NavajoError::new(IoError(err))
    }
}

impl Display for NavajoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.repr {
            IoError(err) => f.write_str(format!("IO error: {}", err).as_str()),
            MessageError { code, message } => f.write_str(format!("Got error: {} {}", code, message).as_str()),
            SocketError { message } => f.write_str(format!("Got error: {}", message).as_str())
        }
    }
}

impl NavajoError {
    pub fn new(repr: NavajoErrorRepr) -> Self {
        Self { repr }
    }
}