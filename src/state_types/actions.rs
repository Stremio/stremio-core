use crate::types::*;
use serde_derive::*;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    // @TODO most of these values need content
    CatalogGrouped { extra: Extra },
    CatalogFiltered,
    Detail,
    Streams,
    AddonCatalog,
}
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        // @TODO map CatalogFilteredto FromAddon
        // @TODO map Detail/Streams to AllOfResource
        // etc.
        match self {
            ActionLoad::CatalogGrouped { extra } => Some(AggrRequest::AllCatalogs {
                extra: extra.to_owned(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login { email: String, password: String },
    Register { email: String, password: String },
    Logout,
    PullAddons,
    PushAddons,
    // @TODO consider PullUser, PushUser?
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "err", content = "args")]
pub enum MiddlewareError {
    API(APIErr),
    Env(String),
    AuthRequired,
}
impl From<APIErr> for MiddlewareError {
    fn from(e: APIErr) -> Self {
        MiddlewareError::API(e)
    }
}
impl From<Box<dyn Error>> for MiddlewareError {
    fn from(e: Box<dyn Error>) -> Self {
        MiddlewareError::Env(e.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    // Input actions
    Load(ActionLoad),
    AddonOp(ActionAddon),

    // user-specific action
    UserOp(ActionUser),

    // Intermediery
    LoadWithUser(Option<User>, Vec<Descriptor>, ActionLoad),
    AddonsChanged(Vec<Descriptor>),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),

    // Output actions
    UserMiddlewareFatal(MiddlewareError),
    UserOpError(ActionUser, MiddlewareError),
    NewState(usize),
}
