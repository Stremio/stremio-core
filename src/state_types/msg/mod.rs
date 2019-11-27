use crate::state_types::{CtxContent, EnvError, SsSettings};
use crate::types::addons::*;
use crate::types::api::*;
use crate::types::{LibBucket, Stream};
use derive_more::*;
use serde_derive::*;

pub mod actions;
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
    StreamingServerSettingsErrored(String),
}

//
// Event
// Those are meant to be user directly by users of the stremio-core crate
//

pub mod ctx_error;
pub use ctx_error::*;

pub mod event;
pub use event::*;

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
