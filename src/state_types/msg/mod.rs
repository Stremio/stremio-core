use crate::types::addons::*;
use crate::types::api::*;
use crate::addon_transport::AddonInterface;
use serde_derive::*;
use std::error::Error;

mod actions;
pub use actions::*;

//
// Intermediery messages
// those are emitted by the middlewares and received by containers
//
#[derive(Debug)]
pub struct Context {
    pub user: Option<User>,
    pub addons: Vec<Descriptor>,
}

pub enum Internal {
    LoadWithCtx(Context, ActionLoad),
    SetInternalAddon(String, Box<dyn AddonInterface>),
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
