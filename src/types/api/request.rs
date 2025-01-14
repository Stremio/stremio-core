use core::fmt;

use crate::constants::{API_URL, LINK_API_URL};
use crate::types::addon::Descriptor;
use crate::types::library::LibraryItem;
use crate::types::profile::{AuthKey, GDPRConsent, Password, User};
use crate::types::resource::SeriesInfo;
use chrono::{DateTime, Local};
#[cfg(test)]
use derivative::Derivative;
use http::Method;
use serde::{Deserialize, Serialize};
use url::Url;

pub trait FetchRequestParams<T> {
    /// Version path prefix for the request
    const VERSION: &'static str = "";

    fn endpoint(&self) -> Url;
    fn method(&self) -> Method;
    fn path(&self) -> String;

    /// Returns the versioned path for the API request.
    ///
    /// In case of v1 we do not have any prefix and the default [`FetchRequestParams::VERSION`] is an empty string.
    ///
    /// V1 path: `create`
    /// V2 path: `v2/create` (where version prefix is `"v2"`)
    fn version_path(&self) -> String {
        if Self::VERSION.is_empty() {
            self.path()
        } else {
            format!(
                "{version}/{path}",
                version = Self::VERSION,
                path = &self.path(),
            )
        }
    }
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
    DeleteAccount {
        auth_key: AuthKey,
        password: Password,
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
    /// Sends Seek log request to the API
    #[serde(rename_all = "camelCase")]
    SeekLog(SeekLogRequest),
    /// Retrieve Skip gaps for Intro and Outro in movie series from the API
    #[serde(rename_all = "camelCase")]
    SkipGaps(SkipGapsRequest),
    #[serde(rename_all = "camelCase")]
    GetModal {
        date: DateTime<Local>,
    },
    #[serde(rename_all = "camelCase")]
    GetNotification {
        date: DateTime<Local>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeekLog {
    /// in milliseconds
    pub from: u64,
    /// in milliseconds
    pub to: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SeekLogRequest {
    /// Opensubtitles hash returned by the server
    #[serde(rename = "osId")]
    pub opensubtitles_hash: String,
    pub item_id: String,
    #[serde(flatten)]
    pub series_info: SeriesInfo,
    /// Stream name hash
    ///
    /// base64 encoded SHA-256 hash of the Stream file name.
    #[serde(rename = "stHash")]
    pub stream_name_hash: String,
    pub duration: u64,
    pub seek_history: Vec<SeekLog>,
    /// The time (in milliseconds) when the user decided to play the next video/episode
    pub skip_outro: Vec<u64>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkipGapsRequest {
    /// User's Auth Key, this requests requires premium account.
    pub auth_key: AuthKey,
    /// Opensubtitles hash returned by the server
    #[serde(rename = "osId")]
    pub opensubtitles_hash: String,
    pub item_id: String,
    #[serde(flatten)]
    pub series_info: SeriesInfo,
    /// Stream name hash
    ///
    /// base64 encoded SHA-256 hash of the Stream file name.
    #[serde(rename = "stHash")]
    pub stream_name_hash: String,
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
            APIRequest::Auth(AuthRequest::Facebook { .. }) => "authWithFacebook".to_owned(),
            APIRequest::Auth(AuthRequest::Register { .. }) => "register".to_owned(),
            APIRequest::Logout { .. } => "logout".to_owned(),
            APIRequest::DeleteAccount { .. } => "deleteUser".to_owned(),
            APIRequest::AddonCollectionGet { .. } => "addonCollectionGet".to_owned(),
            APIRequest::AddonCollectionSet { .. } => "addonCollectionSet".to_owned(),
            APIRequest::GetUser { .. } => "getUser".to_owned(),
            APIRequest::SaveUser { .. } => "saveUser".to_owned(),
            APIRequest::DataExport { .. } => "dataExport".to_owned(),
            APIRequest::Events { .. } => "events".to_owned(),
            APIRequest::SeekLog { .. } => "seekLog".to_owned(),
            APIRequest::SkipGaps { .. } => "getSkipGaps".to_owned(),
            APIRequest::GetModal { .. } => "getModal".to_owned(),
            APIRequest::GetNotification { .. } => "getNotification".to_owned(),
        }
    }
    fn query(&self) -> Option<String> {
        None
    }
    fn body(self) -> APIRequest {
        self
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    Facebook {
        token: String,
    },
    LoginWithToken {
        token: String,
    },
}

impl fmt::Debug for AuthRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login {
                email,
                password: _,
                facebook,
            } => f
                .debug_struct("Login")
                .field("email", email)
                .field("password", &"<SENSITIVE>")
                .field("facebook", facebook)
                .finish(),
            Self::Register {
                email,
                password: _,
                gdpr_consent,
            } => f
                .debug_struct("Register")
                .field("email", email)
                .field("password", &"<SENSITIVE>")
                .field("gdpr_consent", gdpr_consent)
                .finish(),
            Self::Facebook { token: _ } => f
                .debug_struct("Facebook")
                .field("token", &"<SENSITIVE>")
                .finish(),
            Self::LoginWithToken { token: _ } => f
                .debug_struct("LoginWithToken")
                .field("token", &"<SENSITIVE>")
                .finish(),
        }
    }
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
impl LinkRequest {}

impl FetchRequestParams<()> for LinkRequest {
    const VERSION: &'static str = "v2";

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

#[cfg(test)]
mod tests {
    use http::Method;

    use crate::types::api::FetchRequestParams;

    #[test]
    fn test_versioning_of_api_fetch_request_params() {
        struct V2Request;
        impl FetchRequestParams<()> for V2Request {
            const VERSION: &'static str = "v2";
            fn endpoint(&self) -> url::Url {
                "https://example.com/".parse().unwrap()
            }

            fn method(&self) -> Method {
                Method::POST
            }

            fn path(&self) -> String {
                "create".into()
            }

            fn query(&self) -> Option<String> {
                None
            }

            fn body(self) {}
        }

        let v2 = V2Request;
        assert_eq!("v2/create", v2.version_path());
    }
}
