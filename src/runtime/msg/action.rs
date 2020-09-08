use crate::models::addon_details::Selected as AddonDetailsSelected;
use crate::models::catalog_with_filters::Selected as CatalogWithFiltersSelected;
use crate::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::models::library_with_filters::Selected as LibraryWithFiltersSelected;
use crate::models::meta_details::Selected as MetaDetailsSelected;
use crate::models::player::Selected as PlayerSelected;
use crate::models::streaming_server::Settings as StreamingServerSettings;
use crate::types::addon::Descriptor;
use crate::types::api::AuthRequest;
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::MetaItemPreview;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    Authenticate(AuthRequest),
    Logout,
    InstallAddon(Descriptor),
    UninstallAddon(Url),
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

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionStreamingServer {
    Reload,
    UpdateSettings(StreamingServerSettings),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    UpdateLibraryItemState { time: u64, duration: u64 },
    PushToLibrary,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "model", content = "args")]
pub enum ActionLoad {
    Ctx,
    AddonDetails(AddonDetailsSelected),
    CatalogWithFilters(CatalogWithFiltersSelected),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    LibraryWithFilters(LibraryWithFiltersSelected),
    MetaDetails(MetaDetailsSelected),
    Player(PlayerSelected),
    Notifications,
}

//
// Those messages are meant to be dispatched only by the users of the stremio-core crate and handled by the stremio-core crate
//
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Ctx(ActionCtx),
    StreamingServer(ActionStreamingServer),
    Player(ActionPlayer),
    Load(ActionLoad),
    Unload,
}
