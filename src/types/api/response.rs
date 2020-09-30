use crate::types::addon::Descriptor;
use crate::types::profile::{AuthKey, User};
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIError },
    Ok { result: T },
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
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

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct True;

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<True, D::Error>
    where
        D: Deserializer<'de>,
    {
        match bool::deserialize(deserializer) {
            Ok(value) if value => Ok(True),
            Ok(value) => Err(serde::de::Error::invalid_value(
                Unexpected::Bool(value),
                &"true",
            )),
            Err(error) => Err(error),
        }
    }
}

impl Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(true)
    }
}
