use crate::state_types::EnvError;
use crate::types::api::APIError;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum OtherError {
    UserNotLoggedIn,
    LibraryItemNotFound,
    AddonAlreadyInstalled,
    AddonNotInstalled,
    AddonIsProtected,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CtxError {
    API(APIError),
    Env { message: String },
    Other { message: String, code: u64 },
}

impl From<APIError> for CtxError {
    fn from(error: APIError) -> Self {
        CtxError::API(error)
    }
}

impl From<EnvError> for CtxError {
    fn from(error: EnvError) -> Self {
        CtxError::Env {
            message: error.to_string(),
        }
    }
}

impl From<OtherError> for CtxError {
    fn from(error: OtherError) -> Self {
        CtxError::Other {
            code: match &error {
                OtherError::UserNotLoggedIn => 800,
                OtherError::LibraryItemNotFound => 801,
                OtherError::AddonAlreadyInstalled => 802,
                OtherError::AddonNotInstalled => 803,
                OtherError::AddonIsProtected => 804,
            },
            message: match &error {
                OtherError::UserNotLoggedIn => "User is not logged in".to_owned(),
                OtherError::LibraryItemNotFound => "Item is not found in library".to_owned(),
                OtherError::AddonAlreadyInstalled => "Addon is already installed".to_owned(),
                OtherError::AddonNotInstalled => "Addon is not installed".to_owned(),
                OtherError::AddonIsProtected => "Addon is protected".to_owned(),
            },
        }
    }
}
