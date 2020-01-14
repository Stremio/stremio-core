use super::{Action, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserDataRetrivedFromStorage,
    UserLoggedIn,
    UserRegistered,
    UserLoggedOut,
    AddonInstalled,
    AddonUninstalled,
    SettingsUpdated,
    UserDataPersisted,
    LibraryPersisted,
    ActionError(Action, MsgError),
    StorageError(MsgError)
}
