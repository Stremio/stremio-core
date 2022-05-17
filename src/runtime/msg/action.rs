use crate::models::addon_details::Selected as AddonDetailsSelected;
use crate::models::catalog_with_filters::Selected as CatalogWithFiltersSelected;
use crate::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::models::installed_addons_with_filters::Selected as InstalledAddonsWithFiltersSelected;
use crate::models::library_with_filters::Selected as LibraryWithFiltersSelected;
use crate::models::meta_details::Selected as MetaDetailsSelected;
use crate::models::player::Selected as PlayerSelected;
use crate::models::streaming_server::Settings as StreamingServerSettings;
use crate::types::addon::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::MetaItemPreview;
use serde::Deserialize;
use std::ops::Range;

#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    Authenticate(AuthRequest),
    Logout,
    InstallAddon(Descriptor),
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

#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCatalogsWithExtra {
    LoadRange(Range<usize>),
}

#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionStreamingServer {
    Reload,
    UpdateSettings(StreamingServerSettings),
}

#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLink {
    ReadData,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    UpdateLibraryItemState { time: u64, duration: u64 },
    PushToLibrary,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "model", content = "args")]
pub enum ActionLoad {
    AddonDetails(AddonDetailsSelected),
    CatalogWithFilters(CatalogWithFiltersSelected),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    InstalledAddonsWithFilters(InstalledAddonsWithFiltersSelected),
    LibraryWithFilters(LibraryWithFiltersSelected),
    MetaDetails(MetaDetailsSelected),
    Player(PlayerSelected),
    Link,
    Notifications,
}

//
// Those messages are meant to be dispatched only by the users of the stremio-core crate and handled by the stremio-core crate
//
#[derive(Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Ctx(ActionCtx),
    Link(ActionLink),
    CatalogsWithExtra(ActionCatalogsWithExtra),
    StreamingServer(ActionStreamingServer),
    Player(ActionPlayer),
    Load(ActionLoad),
    Unload,
}
