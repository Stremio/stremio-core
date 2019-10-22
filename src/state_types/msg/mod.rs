use crate::state_types::{CtxContent, EnvError, SsSettings, SsError};
use crate::types::addons::*;
use crate::types::api::*;
use crate::types::{LibBucket, Stream};
use serde_derive::*;
use std::error::Error;
use derive_more::*;

mod actions;
pub use actions::*;

//
// Intermediery messages
// those are emitted by the middlewares and received by containers
//
#[derive(Debug)]
pub enum Internal {
    // Context loaded from storage
    CtxLoaded(Option<Box<CtxContent>>),
    // Context updated as a result of authentication/logout
    CtxUpdate(Box<CtxContent>),
    // Context addons pulled
    // this should replace ctx.content.addons entirely
    CtxAddonsPulled(AuthKey, Vec<Descriptor>),
    // Library is loaded, either from storage or from an initial API sync
    // This will replace the whole library index, as long as bucket UID matches
    LibLoaded(LibBucket),
    // Library: pulled some newer items from the API
    // this will extend the current index of libitems, it doesn't replace it
    LibSyncPulled(LibBucket),
    // Response from an add-on
    AddonResponse(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    StreamingServerSettingsLoaded(SsSettings),
    StreamingServerSettingsErrored(SsError),
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
    // This will be used by models which want to re-load the libitem when it may be updated
    // will be emitted after persisting
    LibPersisted,
    LibPushed,
    LibFatal(CtxError),
}

//
// Final enum Msg
// sum type of actions, internals and outputs
//
#[derive(Debug, From)]
pub enum Msg {
    Action(Action),
    Internal(Internal),
    Event(Event),
}

// @TODO separate module
pub enum PlayerProp {
    Time(u64),
    Volume(u8),
    Paused(bool),
}
pub enum PlayerAction {
    GetAllProps,
    SetProp(PlayerProp),
    // by default, all are observed
    //ObserveProp()
    // @TODO should this be PlayerCommand
    Load(Stream),
}

pub enum PlayerEvent {
    PropChanged(PlayerProp),
    PropValue(PlayerProp),
    Loaded, // @TODO: tracks and etc.
    Error,  // @TODO: error type
}
