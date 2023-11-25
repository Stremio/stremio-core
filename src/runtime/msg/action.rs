use std::ops::Range;

use serde::Deserialize;
use url::Url;

use crate::{
    models::{
        addon_details::Selected as AddonDetailsSelected,
        catalog_with_filters::Selected as CatalogWithFiltersSelected,
        catalogs_with_extra::Selected as CatalogsWithExtraSelected,
        installed_addons_with_filters::Selected as InstalledAddonsWithFiltersSelected,
        library_by_type::Selected as LibraryByTypeSelected,
        library_with_filters::Selected as LibraryWithFiltersSelected,
        meta_details::Selected as MetaDetailsSelected,
        player::{Selected as PlayerSelected, VideoParams},
        streaming_server::{
            Settings as StreamingServerSettings,
            StatisticsRequest as StreamingServerStatisticsRequest,
        },
    },
    types::{
        addon::Descriptor,
        api::AuthRequest,
        library::{LibraryItem, LibraryItemId},
        profile::Settings as ProfileSettings,
        resource::{MetaItemId, MetaItemPreview, Video},
    },
};

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    Authenticate(AuthRequest),
    Logout,
    InstallAddon(Descriptor),
    InstallTraktAddon,
    LogoutTrakt,
    UpgradeAddon(Descriptor),
    UninstallAddon(Descriptor),
    UpdateSettings(ProfileSettings),
    AddToLibrary(MetaItemPreview),
    RemoveFromLibrary(String),
    RewindLibraryItem(String),
    /// If boolean is set to `true` it will disable notifications for the LibraryItem.
    ToggleLibraryItemNotifications(LibraryItemId, bool),
    /// Dismiss all Notification for a given [`MetaItemId`].
    DismissNotificationItem(MetaItemId),
    ClearSearchHistory,
    PushUserToAPI,
    PullUserFromAPI,
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
    TimeChanged {
        time: u64,
        duration: u64,
        device: String,
    },
    PausedChanged {
        paused: bool,
    },
    Ended,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "model", content = "args")]
pub enum ActionLoad {
    AddonDetails(AddonDetailsSelected),
    CatalogWithFilters(Option<CatalogWithFiltersSelected>),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    DataExport,
    InstalledAddonsWithFilters(InstalledAddonsWithFiltersSelected),
    LibraryWithFilters(LibraryWithFiltersSelected),
    LibraryByType(LibraryByTypeSelected),
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
    MetaDetails(ActionMetaDetails),
    StreamingServer(ActionStreamingServer),
    Player(ActionPlayer),
    Load(ActionLoad),
    Search(ActionSearch),
    Unload,
}
