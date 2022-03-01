use crate::models::ctx::CtxError;
use crate::types::api::AuthRequest;
use crate::types::profile::{AuthKey, Settings, UID};
use serde::Serialize;
use url::Url;

//
// Those messages are meant to be dispatched by the stremio-core crate and hanled by the users of the stremio-core crate and by the stremio-core crate itself
//
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
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
    LinkCodeCreated { code: Option<String> },
    LinkTokenReceived { token: Option<String> },
    UserLoggedOut { uid: UID },
    SessionDeleted { auth_key: AuthKey },
    AddonInstalled { transport_url: Url, id: String },
    AddonUpgraded { transport_url: Url, id: String },
    AddonUninstalled { transport_url: Url, id: String },
    SettingsUpdated { settings: Settings },
    LibraryItemAdded { id: String },
    LibraryItemRemoved { id: String },
    LibraryItemRewinded { id: String },
    Error { error: CtxError, source: Box<Event> },
}
