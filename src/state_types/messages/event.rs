use super::{Action, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserAuthenticated,
    UserLoggedOut,
    UserDataPersisted,
    LibraryPersisted,
    ActionError(Action, MsgError),
}
