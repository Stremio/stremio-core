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
pub struct APIRequest {
    #[serde(rename = "authKey")]
    pub key: Option<AuthKey>,
    #[serde(flatten)]
    pub body: APIRequestBody,
}
#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum APIRequestBody {
    Login{ email: String, password: String },
    Register{ email: String, password: String },
    Logout,
    AddonCollectionGet,
    AddonCollectionSet{ addons: Vec<Descriptor> },
}
impl APIRequestBody {
    pub fn method_name(&self) -> &str {
        match self {
            APIRequestBody::Login{ .. } => "login",
            APIRequestBody::Register{ .. } => "register",
            APIRequestBody::Logout => "logout",
            APIRequestBody::AddonCollectionGet => "addonCollectionGet",
            APIRequestBody::AddonCollectionSet{ .. } => "addonCollectionSet",
        }
    }
    pub fn requires_auth(&self) -> bool {
        match self {
            APIRequestBody::Logout => true,
            APIRequestBody::AddonCollectionSet{ .. } => true,
            _ => false
        }
    }
}
impl From<&ActionUser> for APIRequestBody {
    fn from(action: &ActionUser) -> Self {
        match action.to_owned() {
            ActionUser::Login{ email, password } => APIRequestBody::Login{ email, password },
            ActionUser::Register{ email, password } => APIRequestBody::Register{ email, password },
            ActionUser::Logout => APIRequestBody::Logout,
            ActionUser::PullAddons => APIRequestBody::AddonCollectionGet,
            // @TODO
            ActionUser::PushAddons => APIRequestBody::AddonCollectionSet{ addons: vec![] },
        }
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
