use crate::state_types::models::ctx::error::CtxError;
use crate::types::addons::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::UID;
use serde::Serialize;
use url::Url;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    ProfilePulledFromStorage {
        uid: UID,
    },
    ProfilePushedToStorage {
        uid: UID,
    },
    UserPulledFromAPI {
        uid: UID,
    },
    UserPushedToAPI {
        uid: UID,
    },
    AddonsPulledFromAPI {
        uid: UID,
    },
    AddonsPushedToAPI {
        uid: UID,
    },
    UserAuthenticated {
        uid: UID,
        auth_request: AuthRequest,
    },
    UserLoggedOut {
        uid: UID,
    },
    SessionDeleted {
        uid: UID,
    },
    AddonInstalled {
        uid: UID,
        addon: Descriptor,
    },
    AddonUninstalled {
        uid: UID,
        addon: Descriptor,
    },
    SettingsUpdated {
        uid: UID,
    },
    LibraryPulledFromStorage {
        uid: UID,
    },
    LibraryPushedToStorage {
        uid: UID,
    },
    LibrarySyncedWithAPI {
        uid: UID,
    },
    LibraryPushedToAPI {
        uid: UID,
    },
    StreamingServerLoaded {
        #[serde(with = "url_serde")]
        url: Url,
    },
    StreamingServerSettingsUpdated {
        #[serde(with = "url_serde")]
        url: Url,
    },
    Error {
        error: CtxError,
        source: Box<Event>,
    },
}
