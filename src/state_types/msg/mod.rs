use crate::state_types::CtxContent;
use crate::types::addons::*;
use crate::types::api::*;
use serde_derive::*;
use std::error::Error;

mod actions;
pub use actions::*;

//
// Intermediery messages
// those are emitted by the middlewares and received by containers
//
pub enum Internal {
    CtxLoaded(Option<Box<CtxContent>>),
    CtxUpdate(Box<CtxContent>),
    CtxAddonsPulled(AuthKey, Vec<Descriptor>),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),
}

//
// Event
// Those are meant to be user directly by users of the stremio-core crate
//
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "err", content = "args")]
pub enum CtxError {
    API(APIErr),
    Env(String),
}
impl From<APIErr> for CtxError {
    fn from(e: APIErr) -> Self {
        CtxError::API(e)
    }
}
impl From<Box<dyn Error>> for CtxError {
    fn from(e: Box<dyn Error>) -> Self {
        CtxError::Env(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    CtxSaved,
    CtxChanged,
    CtxAddonsChangedFromPull,
    CtxAddonsPushed,
    CtxFatal(CtxError),
    CtxActionErr(ActionUser, CtxError),
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
