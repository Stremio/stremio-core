mod user;
pub use self::user::*;

use crate::types::addons::*;
use serde_derive::*;

// @TODO type of error?
#[derive(Deserialize, Clone, Debug)]
pub struct APIErr {
    pub message: String,
}

// @TODO
#[derive(Serialize, Clone)]
pub struct CollectionRequest {}
#[derive(Serialize, Deserialize)]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
}
#[derive(Deserialize)]
#[serde(untagged)]
pub enum APIResult<T> {
    Ok { result: T },
    Err { error: APIErr },
}
