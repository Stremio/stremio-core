use crate::types::addons::*;
use crate::types::api::*;
use serde_derive::*;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    // @TODO most of these values need content
    CatalogGrouped { extra: Vec<ExtraProp> },
    CatalogFiltered,
    Detail { type_name: String, id: String },
    Streams { type_name: String, id: String },
    AddonCatalog,
}
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        // @TODO map CatalogFiltered to FromAddon
        match self {
            ActionLoad::CatalogGrouped { extra } => Some(AggrRequest::AllCatalogs {
                extra: extra.to_owned(),
            }),
            ActionLoad::Detail { type_name, id } => Some(AggrRequest::AllOfResource(
                ResourceRef::without_extra("meta", type_name, id),
            )),
            ActionLoad::Streams { type_name, id } => Some(AggrRequest::AllOfResource(
                ResourceRef::without_extra("stream", type_name, id),
            )),
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
    AuthRace,
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
    LoadWithCtx(Option<User>, Vec<Descriptor>, ActionLoad),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),

    // Output actions
    UserMiddlewareFatal(MiddlewareError),
    UserOpError(ActionUser, MiddlewareError),
    AddonsChanged,
    AddonsChangedFromPull,
    AuthChanged(Option<User>),
    NewState(usize),
}
