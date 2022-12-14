use crate::types::addon::Descriptor;
use crate::types::profile::{AuthKey, User};
use crate::types::True;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use derive_more::TryInto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIError },
    Ok { result: T },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
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

#[derive(Deserialize)]
#[serde(rename = "camelCase")]
pub struct DataExportResponse {
    pub export_id: String,
}

#[derive(PartialEq, Eq, Deserialize, Debug)]

pub struct LibraryItemModified(
    pub String,
    #[serde(with = "ts_milliseconds")] pub DateTime<Utc>,
);

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: True,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]

pub struct LinkCodeResponse {
    pub code: String,
    pub link: String,
    pub qrcode: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkAuthKey {
    pub auth_key: String,
}

#[derive(Clone, TryInto, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum LinkDataResponse {
    AuthKey(LinkAuthKey),
}
