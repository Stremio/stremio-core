use crate::state_types::models::ctx::CtxError;
use crate::types::api::AuthRequest;
use crate::types::profile::{AuthKey, Settings, UID};
use serde::Serialize;
use url::Url;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    CtxPulledFromStorage { uid: UID },
    ProfilePushedToStorage { uid: UID },
    LibraryItemsPushedToStorage { ids: Vec<String> },
    UserPulledFromAPI { uid: UID },
    UserPushedToAPI { uid: UID },
    AddonsPulledFromAPI { transport_urls: Vec<Url> },
    AddonsPushedToAPI { transport_urls: Vec<Url> },
    LibrarySyncWithAPIPlanned { plan: (Vec<String>, Vec<String>) },
    LibraryItemsPushedToAPI { ids: Vec<String> },
    LibraryItemsPulledFromAPI { ids: Vec<String> },
    UserAuthenticated { auth_request: AuthRequest },
    UserLoggedOut { uid: UID },
    SessionDeleted { auth_key: AuthKey },
    AddonInstalled { transport_url: Url },
    AddonUninstalled { transport_url: Url },
    SettingsUpdated { settings: Settings },
    LibraryItemAdded { id: String },
    LibraryItemRemoved { id: String },
    LibraryItemRewided { id: String },
    Error { error: CtxError, source: Box<Event> },
}
