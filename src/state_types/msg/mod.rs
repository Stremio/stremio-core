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

pub mod internal;
pub use internal::*;

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
