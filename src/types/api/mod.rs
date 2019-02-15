mod user;
pub use self::user::*;

use crate::types::addons::*;
use serde_derive::*;

// @TODO type of error?
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct APIErr {
    pub message: String,
}

// @TODO auth_key and stuff
#[derive(Serialize, Clone)]
pub struct CollectionRequest {
    #[serde(rename = "authKey")]
    pub key: String,
}
#[derive(Serialize, Deserialize)]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
}

#[derive(Serialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
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
    Ok { result: T },
    Err { error: APIErr },
}
