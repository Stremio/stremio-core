use super::{ActionCtx, MsgError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
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
    CtxError {
        action_ctx: ActionCtx,
        error: MsgError,
    },
    Error {
        error: MsgError,
    },
}
