use crate::types::addon::Descriptor;
use crate::types::profile::{AuthKey, User};
use crate::types::True;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIError },
    Ok { result: T },
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default, Debug))]
pub struct APIError {
    pub message: String,
    pub code: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "authKey")]
    pub key: AuthKey,
    pub user: User,
}

#[derive(PartialEq, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct LibraryItemModified(
    pub String,
    #[serde(with = "ts_milliseconds")] pub DateTime<Utc>,
);

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: True,
}
