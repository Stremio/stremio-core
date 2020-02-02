use crate::state_types::models::ctx::error::CtxError;
use serde::Serialize;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    ProfilePulledFromStorage,
    ProfilePushedToStorage,
    UserPulledFromAPI,
    UserPushedToAPI,
    AddonsPulledFromAPI,
    AddonsPushedToAPI,
    UserAuthenticated,
    UserLoggedOut,
    SessionDeleted,
    AddonInstalled,
    AddonUninstalled,
    SettingsUpdated,
    LibraryPulledFromStorage,
    LibraryPushedToStorage,
    LibrarySyncedWithAPI,
    LibraryPushedToAPI,
    Error(CtxError),
}
