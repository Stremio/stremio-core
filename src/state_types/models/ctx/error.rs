use crate::state_types::EnvError;
use crate::types::api::APIErr;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CtxError {
    API { message: String, code: u64 },
    Env { message: String },
    Other { message: String },
}

impl fmt::Display for CtxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CtxError::API { message, code } => write!(f, "{} {}", message, code),
            CtxError::Env { message } | CtxError::Other { message } => write!(f, "{}", message),
        }
    }
}

impl Error for CtxError {
    fn description(&self) -> &str {
        match &self {
            CtxError::API { message, .. }
            | CtxError::Env { message }
            | CtxError::Other { message } => message,
        }
    }
}

impl From<APIErr> for CtxError {
    fn from(error: APIErr) -> Self {
        CtxError::API {
            message: error.message.to_owned(),
            code: error.code.to_owned(),
        }
    }
}

impl From<EnvError> for CtxError {
    fn from(error: EnvError) -> Self {
        CtxError::Env {
            message: error.to_string(),
        }
    }
}

impl From<&str> for CtxError {
    fn from(message: &str) -> Self {
        CtxError::Other {
            message: message.to_string(),
        }
    }
}
