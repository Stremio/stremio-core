use crate::state_types::models::ctx::CtxError;
use crate::types::addons::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::profile::UID;
use serde::Serialize;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    CtxPulledFromStorage { uid: UID },
    ProfilePushedToStorage { uid: UID },
    LibraryPushedToStorage { uid: UID },
    UserPulledFromAPI { uid: UID },
    UserPushedToAPI { uid: UID },
    AddonsPulledFromAPI { uid: UID },
    AddonsPushedToAPI { uid: UID },
    LibrarySyncedWithAPI { uid: UID },
    LibraryPushedToAPI { uid: UID },
    UserAuthenticated { uid: UID, auth_request: AuthRequest },
    UserLoggedOut { uid: UID },
    SessionDeleted { uid: UID },
    AddonInstalled { uid: UID, addon: Descriptor },
    AddonUninstalled { uid: UID, addon: Descriptor },
    SettingsUpdated { uid: UID },
    Error { error: CtxError, source: Box<Event> },
}
