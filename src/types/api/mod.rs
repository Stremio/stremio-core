mod user;
pub use self::user::*;
use crate::state_types::*;
use crate::types::addons::*;
use serde_derive::*;

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
    pub fn from_action_with_auth(
        action: &ActionUser,
        key: Option<AuthKey>,
        addons: Vec<Descriptor>,
    ) -> Option<Self> {
        Some(match action.to_owned() {
            ActionUser::Login { email, password } => APIRequest::Login { email, password },
            ActionUser::Register { email, password } => APIRequest::Register { email, password },
            ActionUser::Logout => APIRequest::Logout { auth_key: key? },
            ActionUser::PullAddons => APIRequest::AddonCollectionGet { auth_key: key? },
            ActionUser::PushAddons => APIRequest::AddonCollectionSet {
                auth_key: key?,
                addons: addons.to_owned(),
            },
        })
    }
}

//
// Responses
//
// @TODO type of error?
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct APIErr {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct CollectionResponse {
    pub addons: Vec<Descriptor>,
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
