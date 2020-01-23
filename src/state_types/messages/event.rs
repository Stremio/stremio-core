use super::{Action, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserDataRetrievedFromStorage,
    UserPushedToAPI,
    UserPulledFromAPI,
    UserAuthenticated,
    UserLoggedOut,
    UserSessionDeleted,
    AddonsPushedToAPI,
    AddonsPulledFromAPI,
    AddonInstalled,
    AddonUninstalled,
    SettingsUpdated,
    UserDataPersisted,
    LibraryPersisted,
    LibraryPushed,
    LibrarySynced,
    ActionError(Action, MsgError),
    Error(MsgError),
}
