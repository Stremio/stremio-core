use super::ActionCtx;
use crate::state_types::models::common::ModelError;
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
        error: ModelError,
    },
    Error {
        error: ModelError,
    },
}
