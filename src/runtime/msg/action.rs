use std::ops::Range;

use serde::Deserialize;
use url::Url;

use crate::{
    models::{
        addon_details::Selected as AddonDetailsSelected,
        calendar::Selected as CalendarSelected,
        catalog_with_filters::Selected as CatalogWithFiltersSelected,
        catalogs_with_extra::Selected as CatalogsWithExtraSelected,
        installed_addons_with_filters::Selected as InstalledAddonsWithFiltersSelected,
        library_by_type::Selected as LibraryByTypeSelected,
        library_with_filters::Selected as LibraryWithFiltersSelected,
        meta_details::Selected as MetaDetailsSelected,
        player::{Selected as PlayerSelected, VideoParams},
    },
    types::{
        addon::Descriptor,
        api::AuthRequest,
        library::LibraryItemId,
        profile::{AuthKey, Password, Settings as ProfileSettings},
        rating::Rating,
        resource::{MetaItemId, MetaItemPreview, Video},
        streaming_server::{
            Settings as StreamingServerSettings,
            StatisticsRequest as StreamingServerStatisticsRequest,
        },
        streams::StreamItemState,
    },
};

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    Authenticate(AuthRequest),
    Logout,
    DeleteAccount(Password),
    InstallAddon(Descriptor),
    InstallTraktAddon,
    LogoutTrakt,
    UpgradeAddon(Descriptor),
    UninstallAddon(Descriptor),
    UpdateSettings(ProfileSettings),
    AddToLibrary(MetaItemPreview),
    RemoveFromLibrary(String),
    RewindLibraryItem(String),
    LibraryItemMarkAsWatched {
        id: LibraryItemId,
        is_watched: bool,
    },
    /// If boolean is set to `true` it will disable notifications for the LibraryItem.
    ToggleLibraryItemNotifications(LibraryItemId, bool),
    /// Dismiss all Notification for a given [`MetaItemId`].
    DismissNotificationItem(MetaItemId),
    ClearSearchHistory,
    PushUserToAPI,
    /// # Examples
    ///
    /// ```
    /// use stremio_core::runtime::msg::ActionCtx;
    ///
    /// let pull_user_with_token = serde_json::from_value::<ActionCtx>(serde_json::json!({
    ///     "action": "PullUserFromAPI",
    ///     "args": {
    ///         "token": "exampleToken1234",
    ///     },
    /// })).expect("Should be a valid action");
    ///
    /// let pull_user_with_profile_auth = serde_json::from_value::<ActionCtx>(serde_json::json!({
    ///     "action": "PullUserFromAPI",
    ///     "args": {},
    /// })).expect("Should be a valid action");
    /// ```
    PullUserFromAPI {
        /// Optional auth token of the user to be fetched
        ///
        /// if `None` is provided, the `ctx.profile.auth`
        /// will be used to pull the user (if one exists)
        #[serde(default)]
        token: Option<AuthKey>,
    },
    PushAddonsToAPI,
    PullAddonsFromAPI,
    SyncLibraryWithAPI,
    /// Pull notifications for all [`LibraryItem`]s that we should pull notifications for.
    ///
    /// **Warning:** The action will **always** trigger requests to the addons.
    ///
    /// See `LibraryItem::should_pull_notifications()`
    ///
    /// [`LibraryItem`]: crate::types::library::LibraryItem
    PullNotifications,
    /// Make request to api to get events modal and notification
    GetEvents,
    /// Dismiss an event by id, either a Modal or Notification
    DismissEvent(String),
    /// Add a server URL to the list of available streaming servers
    AddServerUrl(Url),
    /// Delete a server URL from the list of available streaming servers
    DeleteServerUrl(Url),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionCatalogWithFilters {
    LoadNextPage,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionCatalogsWithExtra {
    LoadRange(Range<usize>),
    LoadNextPage(usize),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionLibraryByType {
    LoadNextPage(usize),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionLibraryWithFilters {
    LoadNextPage,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionMetaDetails {
    /// Marks the [`LibraryItem`] as watched.
    ///
    /// Applicable when you have single-video (e.g. a movie) and multi-video (e.g. a movie series) item.
    ///
    /// [`LibraryItem`]: crate::types::library::LibraryItem
    MarkAsWatched(bool),
    /// Marks the given [`Video`] of the [`LibraryItem`] as watched.
    ///
    /// Applicable only when you have a multi-video (e.g. movie series) item.
    ///
    /// [`LibraryItem`]: crate::types::library::LibraryItem
    MarkVideoAsWatched(Video, bool),
    /// Mark all videos from given season as watched
    MarkSeasonAsWatched(u32, bool),
    /// Rate the current meta item
    Rate(Option<Rating>),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum CreateTorrentArgs {
    File(Vec<u8>),
    Magnet(Url),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayOnDeviceArgs {
    pub device: String,
    pub source: String,
    pub time: Option<u64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionStreamingServer {
    Reload,
    UpdateSettings(StreamingServerSettings),
    CreateTorrent(CreateTorrentArgs),
    GetStatistics(StreamingServerStatisticsRequest),
    PlayOnDevice(PlayOnDeviceArgs),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionLink {
    ReadData,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    #[serde(rename_all = "camelCase")]
    VideoParamsChanged {
        video_params: Option<VideoParams>,
    },
    StreamStateChanged {
        state: StreamItemState,
    },
    /// Seek performed by the user when using the seekbar or
    /// the shortcuts for seeking.
    ///
    /// When transitioning from Seek to TimeChanged and vice-versa
    /// we need to make sure to update the other accordingly
    /// if we have any type of throttling of these events,
    /// otherwise we will get wrong `LibraryItem.state.time_offset`!
    Seek {
        time: u64,
        duration: u64,
        device: String,
    },
    /// A normal playback by the video player
    ///
    /// The time from one TimeChanged action to another can only grow (move forward)
    /// and should never go backwards, except when a [`ActionPlayer::Seek`] happen
    /// and moves the time backwards.
    TimeChanged {
        time: u64,
        duration: u64,
        device: String,
    },
    PausedChanged {
        paused: bool,
    },
    /// User has clicked on the next video button.
    NextVideo,
    /// Video player has ended.
    ///
    /// 2 scenarios are possible:
    /// - We've watched a movie to the last second
    /// - We've watched a movie series to the last second
    Ended,
    /// Marks the given [`Video`] of the [`LibraryItem`] as watched.
    ///
    /// Applicable only when you have a multi-video (e.g. movie series) item.
    ///
    /// [`LibraryItem`]: crate::types::library::LibraryItem
    MarkVideoAsWatched(Video, bool),
    /// Mark all videos from given season as watched
    MarkSeasonAsWatched(u32, bool),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "model", content = "args")]
/// Action to load a specific Model.
pub enum ActionLoad {
    AddonDetails(AddonDetailsSelected),
    CatalogWithFilters(Option<CatalogWithFiltersSelected>),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    DataExport,
    InstalledAddonsWithFilters(InstalledAddonsWithFiltersSelected),
    LibraryWithFilters(LibraryWithFiltersSelected),
    LibraryByType(LibraryByTypeSelected),
    /// Loads the Calendar Model
    Calendar(Option<CalendarSelected>),
    /// Loads the data required for Local search
    LocalSearch,
    MetaDetails(MetaDetailsSelected),
    Player(Box<PlayerSelected>),
    Link,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionSearch {
    /// Request for Search queries
    #[serde(rename_all = "camelCase")]
    Search {
        search_query: String,
        max_results: usize,
    },
}

/// Action messages
///
/// Those messages are meant to be dispatched only by the users of the
/// `stremio-core` crate and handled by the `stremio-core` crate.
#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Ctx(ActionCtx),
    Link(ActionLink),
    CatalogWithFilters(ActionCatalogWithFilters),
    CatalogsWithExtra(ActionCatalogsWithExtra),
    LibraryByType(ActionLibraryByType),
    LibraryWithFilters(ActionLibraryWithFilters),
    MetaDetails(ActionMetaDetails),
    StreamingServer(ActionStreamingServer),
    Player(ActionPlayer),
    Load(ActionLoad),
    Search(ActionSearch),
    Unload,
}
