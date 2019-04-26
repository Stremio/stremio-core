mod user;
pub use self::user::*;
use crate::types::addons::*;
use chrono::{DateTime, Utc};
use serde::de;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::*;
use std::fmt;

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
// @TODO perhaps we can translate this u64 to an enum
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct APIErr {
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
    pub key: String,
    pub user: User,
}

// Sometimes, the API returns {success: true} as a result
// @TODO find a way to enforce only being able to deserialize `true`
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: True,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIErr },
    Ok { result: T },
}

// Always true
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct True;

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<True, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TrueVisitor;

        impl<'de> Visitor<'de> for TrueVisitor {
            type Value = True;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("true")
            }

            fn visit_bool<E>(self, value: bool) -> Result<True, E>
            where
                E: de::Error,
            {
                match value {
                    true => Ok(True),
                    false => Err(E::invalid_value(Unexpected::Bool(value), &self)),
                }
            }
        }

        deserializer.deserialize_bool(TrueVisitor)
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
