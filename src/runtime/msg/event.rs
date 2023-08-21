use serde::Serialize;
use url::Url;

use crate::{
    models::{ctx::CtxError, player::AnalyticsContext as PlayerAnalyticsContext},
    types::{
        addon::TransportUrl,
        api::AuthRequest,
        library::LibraryItemId,
        profile::{AuthKey, Settings, UID},
    },
};

///
/// Those messages are meant to be dispatched by the `stremio-core` crate and
/// handled by the users of the `stremio-core` crate and by the `stremio-core`
/// crate itself.
#[derive(Clone, Serialize, Debug, PartialEq)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    PlayerPlaying {
        context: PlayerAnalyticsContext,
        load_time: i64,
    },
    PlayerStopped {
        context: PlayerAnalyticsContext,
    },
    PlayerEnded {
        context: PlayerAnalyticsContext,
        is_binge_enabled: bool,
        is_playing_next_video: bool,
    },
    TraktPlaying {
        context: PlayerAnalyticsContext,
    },
    TraktPaused {
        context: PlayerAnalyticsContext,
    },
    ProfilePushedToStorage {
        uid: UID,
    },
    LibraryItemsPushedToStorage {
        ids: Vec<String>,
    },
    StreamsPushedToStorage {
        uid: UID,
    },
    NotificationsPushedToStorage {
        ids: Vec<String>,
    },
    UserPulledFromAPI {
        uid: UID,
    },
    UserPushedToAPI {
        uid: UID,
    },
    AddonsPulledFromAPI {
        transport_urls: Vec<TransportUrl>,
    },
    AddonsPushedToAPI {
        transport_urls: Vec<TransportUrl>,
    },
    LibrarySyncWithAPIPlanned {
        uid: UID,
        plan: (Vec<String>, Vec<String>),
    },
    LibraryItemsPushedToAPI {
        ids: Vec<String>,
    },
    LibraryItemsPulledFromAPI {
        ids: Vec<String>,
    },
    UserAuthenticated {
        auth_request: AuthRequest,
    },
    UserLoggedOut {
        uid: UID,
    },
    SessionDeleted {
        auth_key: AuthKey,
    },
    TraktAddonFetched {
        uid: UID,
    },
    TraktLoggedOut {
        uid: UID,
    },
    AddonInstalled {
        transport_url: TransportUrl,
        id: String,
    },
    AddonUpgraded {
        transport_url: TransportUrl,
        id: String,
    },
    AddonUninstalled {
        transport_url: TransportUrl,
        id: String,
    },
    SettingsUpdated {
        settings: Settings,
    },
    LibraryItemAdded {
        id: LibraryItemId,
    },
    LibraryItemRemoved {
        id: LibraryItemId,
    },
    LibraryItemRewinded {
        id: LibraryItemId,
    },
    LibraryItemNotificationsToggled {
        id: LibraryItemId,
    },
    /// The notifications for the given LibraryItemId have been dismissed
    NotificationsDismissed {
        id: LibraryItemId,
    },
    MagnetParsed {
        magnet: Url,
    },
    TorrentParsed {
        torrent: Vec<u8>,
    },
    PlayingOnDevice {
        device: String,
    },
    Error {
        error: CtxError,
        source: Box<Event>,
    },
}
