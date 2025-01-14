use crate::models::ctx::CtxError;
use crate::models::player::AnalyticsContext as PlayerAnalyticsContext;
use crate::types::api::AuthRequest;
use crate::types::library::LibraryItemId;
use crate::types::profile::{AuthKey, Settings, UID};
use serde::Serialize;
use url::Url;

///
/// Those messages are meant to be dispatched by the `stremio-core` crate and
/// handled by the users of the `stremio-core` crate and by the `stremio-core`
/// crate itself.
#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(tag = "event", content = "args")]
pub enum Event {
    PlayerPlaying {
        context: PlayerAnalyticsContext,
        load_time: i64,
    },
    PlayerStopped {
        context: PlayerAnalyticsContext,
    },
    PlayerNextVideo {
        context: PlayerAnalyticsContext,
        is_binge_enabled: bool,
        is_playing_next_video: bool,
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
    SearchHistoryPushedToStorage {
        uid: UID,
    },
    NotificationsPushedToStorage {
        ids: Vec<String>,
    },
    DismissedEventsPushedToStorage {
        uid: UID,
    },
    UserPulledFromAPI {
        uid: UID,
    },
    UserPushedToAPI {
        uid: UID,
    },
    AddonsPulledFromAPI {
        transport_urls: Vec<Url>,
    },
    AddonsPushedToAPI {
        transport_urls: Vec<Url>,
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
    UserAddonsLocked {
        addons_locked: bool,
    },
    UserLibraryMissing {
        library_missing: bool,
    },
    UserLoggedOut {
        uid: UID,
    },
    UserAccountDeleted {
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
        transport_url: Url,
        id: String,
    },
    AddonUpgraded {
        transport_url: Url,
        id: String,
    },
    AddonUninstalled {
        transport_url: Url,
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
    /// The LibraryItem with the given id has been marked as watched or unwatched (Overrides the previous watched state)
    LibraryItemMarkedAsWatched {
        id: LibraryItemId,
        is_watched: bool,
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
    StreamingServerUrlsBucketChanged {
        uid: UID,
    },
    StreamingServerUrlsPushedToStorage {
        uid: UID,
    },
    Error {
        error: CtxError,
        source: Box<Event>,
    },
}
