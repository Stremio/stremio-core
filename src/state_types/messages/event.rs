use super::{ActionLibrary, ActionUser, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserDataSaved,
    UserDataChanged,
    StorageError(MsgError),
    LibraryActionError(ActionLibrary, MsgError),
    UserActionError(ActionUser, MsgError),
    CtxAddonsChangedFromPull,
    CtxAddonsPushed,
    LibPersisted,
    LibPushed,
    SettingsStoreError(String),
}
