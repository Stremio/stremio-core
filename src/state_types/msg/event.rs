use crate::state_types::models::ctx::error::CtxError;
use serde::Serialize;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserPulledFromStorage,
    UserPushedToStorage,
    UserPulledFromAPI,
    UserPushedToAPI,
    AddonsPushedToAPI,
    AddonsPulledFromAPI,
    UserAuthenticated,
    UserSessionDeleted,
    UserLoggedOut,
    AddonInstalled,
    AddonUninstalled,
    SettingsUpdated,
    LibraryPulledFromStorage,
    LibraryPushedToStorage,
    LibraryPushedToAPI,
    LibrarySyncedWithAPI,
    Error(CtxError),
}
