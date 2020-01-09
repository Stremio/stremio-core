use crate::state_types::EnvError;
use crate::types::api::APIErr;
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "args")]
pub enum MsgError {
    API { message: String, code: u64 },
    Env { message: String },
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
