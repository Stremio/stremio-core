use crate::types::addons::*;
use crate::types::api::*;
use serde_derive::*;
use std::error::Error;

//
// Input actions: those are triggered by users
//
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

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Load(ActionLoad),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
}

//
// Intermediery messages
// those are emitted by the middlewares and received by containers
//
#[derive(Debug)]
pub struct Context {
    pub user: Option<User>,
    pub addons: Vec<Descriptor>,
}

#[derive(Debug)]
pub enum Internal {
    LoadWithCtx(Context, ActionLoad),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),
}

//
// Event
// Those are meant to be user directly by users of the stremio-core crate
//
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

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    ContextMiddlewareFatal(MiddlewareError),
    UserOpError(ActionUser, MiddlewareError),
    AddonsChanged,
    AddonsChangedFromPull,
    AuthChanged(Option<User>),
}

//
// Final enum Msg
// sum type of actions, internals and outputs
//
#[derive(Debug)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}
impl From<Action> for Msg {
    fn from(a: Action) -> Self {
        Msg::Action(a)
    }
}
impl From<Internal> for Msg {
    fn from(i: Internal) -> Self {
        Msg::Internal(i)
    }
}
impl From<Event> for Msg {
    fn from(o: Event) -> Self {
        Msg::Event(o)
    }
}
