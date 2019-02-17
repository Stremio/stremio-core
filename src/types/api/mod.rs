mod user;
pub use self::user::*;
use crate::types::addons::*;
use serde_derive::*;

//
// Requests
//
pub type AuthKey = String;

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum APIRequest {
    Login {
        email: String,
        password: String,
    },
    Register {
        email: String,
        password: String,
    },
    #[serde(rename_all = "camelCase")]
    Logout {
        auth_key: AuthKey,
    },
    #[serde(rename_all = "camelCase")]
    AddonCollectionGet {
        auth_key: AuthKey,
    },
    #[serde(rename_all = "camelCase")]
    AddonCollectionSet {
        auth_key: AuthKey,
        addons: Vec<Descriptor>,
    },
}
impl APIRequest {
    pub fn method_name(&self) -> &str {
        match self {
            APIRequest::Login { .. } => "login",
            APIRequest::Register { .. } => "register",
            APIRequest::Logout { .. } => "logout",
            APIRequest::AddonCollectionGet { .. } => "addonCollectionGet",
            APIRequest::AddonCollectionSet { .. } => "addonCollectionSet",
        }
    }
}

//
// Responses
//
// @TODO type of error?
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct APIErr {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "authKey")]
    pub key: String,
    pub user: User,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIErr },
    Ok { result: T },
}
