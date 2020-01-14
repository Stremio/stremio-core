use super::{Action, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserDataRetrivedFromStorage,
    UserPushedToAPI,
    UserPulledFromAPI,
    UserLoggedIn,
    UserRegistered,
    UserLoggedOut,
    AddonsPushedToAPI,
    AddonsPulledFromAPI,
    AddonInstalled,
    AddonUninstalled,
    SettingsUpdated,
    UserDataPersisted,
    LibraryPersisted,
    StorageError(MsgError),
    ActionError(Action, MsgError),
}
