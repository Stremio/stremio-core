use crate::state_types::models::common::ModelError;
use serde::Serialize;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    UserRetrievedFromStorage,
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
    UserPersisted,
    LibraryRetrievedFromStorage,
    LibraryPersisted,
    LibraryPushedToAPI,
    LibrarySyncedWithAPI,
    StreamingServerLoaded,
    Error(ModelError),
}
