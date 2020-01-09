use crate::state_types::EnvError;
use crate::types::api::APIErr;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "args")]
pub enum MsgError {
    API { message: String, code: u64 },
    Env { message: String },
}

impl fmt::Display for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match &self {
            MsgError::API { message, .. } => message.to_owned(),
            MsgError::Env { message } => message.to_owned(),
        };
        write!(f, "{}", message)
    }
}

impl Error for MsgError {
    fn description(&self) -> &str {
        format!("{}", &self).as_str()
    }
}

impl From<APIErr> for MsgError {
    fn from(error: APIErr) -> Self {
        MsgError::API {
            message: error.message.to_owned(),
            code: error.code.to_owned(),
        }
    }
}

impl From<EnvError> for MsgError {
    fn from(error: EnvError) -> Self {
        MsgError::Env {
            message: error.to_string(),
        }
    }
}
