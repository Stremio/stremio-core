use crate::state_types::models::common::ModelError;
use serde::Serialize;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
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
    LibraryRetrievedFromStorage,
    LibraryPersisted,
    LibraryPushedToAPI,
    LibrarySyncedWithAPI,
    Error(ModelError),
}
