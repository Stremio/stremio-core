use crate::types::addon::Descriptor;
use crate::types::library::LibraryItem;
use crate::types::profile::{AuthKey, GDPRConsent};
#[cfg(test)]
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

pub trait APIMethodName {
    fn method_name(&self) -> &str;
}

#[derive(Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(tag = "type")]
pub enum APIRequest {
    Auth(AuthRequest),
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
            APIRequest::Auth(AuthRequest::Login { .. }) => "login",
            APIRequest::Auth(AuthRequest::Register { .. }) => "register",
            APIRequest::Logout { .. } => "logout",
            APIRequest::AddonCollectionGet { .. } => "addonCollectionGet",
            APIRequest::AddonCollectionSet { .. } => "addonCollectionSet",
        }
    }
}

#[derive(Derivative, Clone, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[cfg_attr(test, derive(Debug))]
#[serde(tag = "type")]
pub enum AuthRequest {
    #[derivative(Default)]
    Login { email: String, password: String },
    Register {
        email: String,
        password: String,
        gdpr_consent: GDPRConsentRequest,
    },
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Derivative, Debug))]
#[cfg_attr(test, derivative(Default))]
pub struct GDPRConsentRequest {
    #[serde(flatten)]
    pub gdpr_consent: GDPRConsent,
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp(0, 0)")))]
    pub time: DateTime<Utc>,
    pub from: String,
}

#[derive(Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct DatastoreRequest {
    pub auth_key: AuthKey,
    pub collection: String,
    #[serde(flatten)]
    pub command: DatastoreCommand,
}

impl APIMethodName for DatastoreRequest {
    fn method_name(&self) -> &str {
        match &self.command {
            DatastoreCommand::Meta => "datastoreMeta",
            DatastoreCommand::Get { .. } => "datastoreGet",
            DatastoreCommand::Put { .. } => "datastorePut",
        }
    }
}

#[derive(Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum DatastoreCommand {
    Meta,
    Get {
        #[serde(default)]
        ids: Vec<String>,
        all: bool,
    },
    Put {
        #[serde(default)]
        changes: Vec<LibraryItem>,
    },
}
