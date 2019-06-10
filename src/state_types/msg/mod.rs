use crate::addon_transport::AddonInterface;
use crate::state_types::Context;
use crate::state_types::CtxContent;
use crate::types::addons::*;
use crate::types::api::*;
use serde_derive::*;
use std::error::Error;
use std::rc::Rc;

mod actions;
pub use actions::*;

//
// Intermediery messages
// those are emitted by the middlewares and received by containers
//
pub enum Internal {
    CtxLoaded(Option<Box<CtxContent>>),
    CtxUpdate(Box<CtxContent>),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),

    // @TODO drop those
    LoadWithCtx(Context, ActionLoad),
    SetInternalAddon(String, Rc<dyn AddonInterface>),
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
    LibIdx,
}
impl From<APIErr> for MiddlewareError {
    fn from(e: APIErr) -> Self {
        MiddlewareError::API(e)
    }
}
// @TODO single impl for those
// the problem is conflicting implementations: https://users.rust-lang.org/t/sized-error-with-box-error/22748
impl From<Box<dyn Error>> for MiddlewareError {
    fn from(e: Box<dyn Error>) -> Self {
        MiddlewareError::Env(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    CtxSaved,
    CtxChanged,
    // @TODO: change ContextMiddlewareFatal to CtxFatal
    ContextMiddlewareFatal(MiddlewareError),
    UserOpError(ActionUser, MiddlewareError),
    AddonsChanged,
    AddonsChangedFromPull,
    AuthChanged(Option<User>),
    LibSynced { pushed: u64, pulled: u64 },
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
