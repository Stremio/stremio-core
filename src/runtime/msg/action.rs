use crate::models::addon_details::Selected as AddonDetailsSelected;
use crate::models::catalog_with_filters::Selected as CatalogWithFiltersSelected;
use crate::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::models::installed_addons_with_filters::Selected as InstalledAddonsWithFiltersSelected;
use crate::models::library_by_type::Selected as LibraryByTypeSelected;
use crate::models::library_with_filters::Selected as LibraryWithFiltersSelected;
use crate::models::meta_details::Selected as MetaDetailsSelected;
use crate::models::player::Selected as PlayerSelected;
use crate::models::streaming_server::{
    Settings as StreamingServerSettings, StatisticsRequest as StreamingServerStatisticsRequest,
};
use crate::types::addon::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::MetaItemPreview;
use serde::Deserialize;
use std::ops::Range;
use url::Url;

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
    PushUserToAPI,
    PullUserFromAPI,
    PushAddonsToAPI,
    PullAddonsFromAPI,
    SyncLibraryWithAPI,
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
    MarkAsWatched(bool),
    MarkVideoAsWatched(String, bool),
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
    MetaDetails(MetaDetailsSelected),
    Player(Box<PlayerSelected>),
    Link,
    Notifications,
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
    Unload,
}
