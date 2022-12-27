use crate::constants::{API_URL, LINK_API_URL};
use crate::types::addon::Descriptor;
use crate::types::library::LibraryItem;
use crate::types::profile::{AuthKey, GDPRConsent, User};
#[cfg(test)]
use derivative::Derivative;
use http::Method;
use serde::{Deserialize, Serialize};
use url::Url;

pub trait FetchRequestParams<T> {
    fn endpoint(&self) -> Url;
    fn method(&self) -> Method;
    fn path(&self) -> String;
    fn query(&self) -> Option<String>;
    fn body(self) -> T;
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
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
    #[serde(rename_all = "camelCase")]
    GetUser {
        auth_key: AuthKey,
    },
    #[serde(rename_all = "camelCase")]
    SaveUser {
        auth_key: AuthKey,
        #[serde(flatten)]
        user: User,
    },
    #[serde(rename_all = "camelCase")]
    DataExport {
        auth_key: AuthKey,
    },
    #[serde(rename_all = "camelCase")]
    Events {
        auth_key: AuthKey,
        events: Vec<serde_json::Value>,
    },
}

impl FetchRequestParams<APIRequest> for APIRequest {
    fn endpoint(&self) -> Url {
        API_URL.to_owned()
    }
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> String {
        match self {
            APIRequest::Auth(AuthRequest::Login { .. }) => "login".to_owned(),
            APIRequest::Auth(AuthRequest::LoginWithToken { .. }) => "loginWithToken".to_owned(),
            APIRequest::Auth(AuthRequest::Register { .. }) => "register".to_owned(),
            APIRequest::Logout { .. } => "logout".to_owned(),
            APIRequest::AddonCollectionGet { .. } => "addonCollectionGet".to_owned(),
            APIRequest::AddonCollectionSet { .. } => "addonCollectionSet".to_owned(),
            APIRequest::GetUser { .. } => "getUser".to_owned(),
            APIRequest::SaveUser { .. } => "saveUser".to_owned(),
            APIRequest::DataExport { .. } => "dataExport".to_owned(),
            APIRequest::Events { .. } => "events".to_owned(),
        }
    }
    fn query(&self) -> Option<String> {
        None
    }
    fn body(self) -> APIRequest {
        self
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(tag = "type")]
pub enum AuthRequest {
    #[cfg_attr(test, derivative(Default))]
    Login {
        email: String,
        password: String,
        #[serde(default)]
        facebook: bool,
    },
    Register {
        email: String,
        password: String,
        gdpr_consent: GDPRConsent,
    },
    LoginWithToken {
        token: String,
    },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(tag = "type")]
pub enum LinkRequest {
    #[cfg_attr(test, derivative(Default))]
    Create,
    Read {
        code: String,
    },
}

impl FetchRequestParams<()> for LinkRequest {
    fn endpoint(&self) -> Url {
        LINK_API_URL.to_owned()
    }
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> String {
        match self {
            LinkRequest::Create => "create".to_owned(),
            LinkRequest::Read { .. } => "read".to_owned(),
        }
    }
    fn query(&self) -> Option<String> {
        Some(serde_url_params::to_string(&self).expect("Serialize query params failed"))
    }
    fn body(self) {}
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DatastoreRequest {
    pub auth_key: AuthKey,
    pub collection: String,
    #[serde(flatten)]
    pub command: DatastoreCommand,
}

impl FetchRequestParams<DatastoreRequest> for DatastoreRequest {
    fn endpoint(&self) -> Url {
        API_URL.to_owned()
    }
    fn method(&self) -> Method {
        Method::POST
    }
    fn path(&self) -> String {
        match &self.command {
            DatastoreCommand::Meta => "datastoreMeta".to_owned(),
            DatastoreCommand::Get { .. } => "datastoreGet".to_owned(),
            DatastoreCommand::Put { .. } => "datastorePut".to_owned(),
        }
    }
    fn query(&self) -> Option<String> {
        None
    }
    fn body(self) -> DatastoreRequest {
        self
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(untagged)]
pub enum DatastoreCommand {
    #[cfg_attr(test, derivative(Default))]
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
