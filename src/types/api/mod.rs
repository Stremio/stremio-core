mod user;
pub use self::user::*;
mod fetch;
pub use self::fetch::*;
mod auth;
pub use self::auth::*;
use crate::types::addons::*;
use chrono::{DateTime, Utc};
use derive_builder::*;
use serde::de;
use serde::de::{Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use serde::{Deserialize, Deserializer};
use serde_derive::*;
use std::fmt;

//
// Requests
//
pub type AuthKey = String;

pub trait APIMethodName {
    fn method_name(&self) -> &str;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GDPRConsent {
    pub tos: bool,
    pub privacy: bool,
    pub marketing: bool,
    pub time: DateTime<Utc>,
    pub from: String,
}

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
        gdpr_consent: GDPRConsent,
    },
    #[serde(rename_all = "camelCase")]
    Logout {
        auth_key: AuthKey,
    },
    #[serde(rename_all = "camelCase")]
    AddonCollectionGet {
        auth_key: AuthKey,
        update: bool,
    },
    #[serde(rename_all = "camelCase")]
    AddonCollectionSet {
        auth_key: AuthKey,
        addons: Vec<Descriptor>,
    },
}
impl APIMethodName for APIRequest {
    fn method_name(&self) -> &str {
        match self {
            APIRequest::Login { .. } => "login",
            APIRequest::Register { .. } => "register",
            APIRequest::Logout { .. } => "logout",
            APIRequest::AddonCollectionGet { .. } => "addonCollectionGet",
            APIRequest::AddonCollectionSet { .. } => "addonCollectionSet",
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum DatastoreCmd {
    Get {
        #[serde(default)]
        ids: Vec<String>,
        all: bool,
    },
    Meta {},
    Put {
        #[serde(default)]
        // NOTE: what if we need to be generic here?
        // we can use separate structs rather than enums, and implement method_name on all of them
        // via a trait; that way, the struct will be generic
        changes: Vec<crate::types::LibItem>,
    },
}

#[derive(Serialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct DatastoreReq {
    auth_key: AuthKey,
    collection: String,
    #[serde(flatten)]
    cmd: DatastoreCmd,
}
impl DatastoreReqBuilder {
    pub fn with_cmd(&mut self, cmd: DatastoreCmd) -> DatastoreReq {
        self.cmd(cmd).build().expect("builder cannot fail")
    }
}
impl APIMethodName for DatastoreReq {
    fn method_name(&self) -> &str {
        match &self.cmd {
            DatastoreCmd::Get { .. } => "datastoreGet",
            DatastoreCmd::Meta {} => "datastoreMeta",
            DatastoreCmd::Put { .. } => "datastorePut",
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
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: True,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum APIResult<T> {
    Err { error: APIErr },
    Ok { result: T },
}

// Always true
// consider moving into a separate module
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
                if value {
                    Ok(True)
                } else {
                    Err(E::invalid_value(Unexpected::Bool(value), &self))
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
