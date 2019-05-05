use crate::types::addons::*;
use crate::types::api::*;
use serde_derive::*;
use std::error::Error;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogGrouped { extra: Vec<ExtraProp> },
    CatalogFiltered { resource_req: ResourceRequest },
    Detail { type_name: String, id: String },
    Streams { type_name: String, id: String },
    // @TODO most of these values need content
    AddonCatalog,
}
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        match self {
            ActionLoad::CatalogGrouped { extra } => Some(AggrRequest::AllCatalogs {
                extra: extra.to_owned(),
            }),
            ActionLoad::CatalogFiltered { resource_req } => {
                Some(AggrRequest::FromAddon(resource_req.to_owned()))
            }
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

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Serialize, Clone)]
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
pub struct Context {
    pub user: Option<User>,
    pub addons: Vec<Descriptor>,
}

// @TODO alternatively, this could just be an enum of Msg { Action, Internal, Event }
// that does not implement Serialize/Deserialize; and we only do so for the inner values
// if we do that, the From traits will be targeting Msg and all functions will be taking Msg
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    // Input actions
    #[serde(skip_serializing)]
    Load(ActionLoad),
    #[serde(skip_serializing)]
    AddonOp(ActionAddon),
    #[serde(skip_serializing)]
    UserOp(ActionUser),

    // Intermediery
    #[serde(skip)]
    LoadWithCtx(Context, ActionLoad),
    #[serde(skip)]
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),

    // Output actions
    #[serde(skip_deserializing)]
    ContextMiddlewareFatal(MiddlewareError),
    #[serde(skip_deserializing)]
    UserOpError(ActionUser, MiddlewareError),
    #[serde(skip_deserializing)]
    AddonsChanged,
    #[serde(skip_deserializing)]
    AddonsChangedFromPull,
    #[serde(skip_deserializing)]
    AuthChanged(Option<User>),
}

