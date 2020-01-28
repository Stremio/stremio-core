use crate::state_types::models::addon_details::Selected as AddonDetailsSelected;
use crate::state_types::models::catalog_filtered::Selected as CatalogFilteredSelected;
use crate::state_types::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::state_types::models::ctx::Settings;
use crate::state_types::models::library_filtered::Selected as LibraryFilteredSelected;
use crate::state_types::models::meta_details::Selected as MetaDetailsSelected;
use crate::state_types::models::player::Selected as PlayerSelected;
use crate::types::addons::{Descriptor, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::MetaPreview;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionAuth {
    Login {
        email: String,
        password: String,
    },
    Register {
        email: String,
        password: String,
        gdpr_consent: GDPRConsent,
    },
    Logout,
    PushToAPI,
    PullFromAPI,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionAddons {
    Install(Descriptor),
    Uninstall(TransportUrl),
    PushToAPI,
    PullFromAPI,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionSettings {
    Update(Settings),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLibrary {
    Add(MetaPreview),
    Remove(String),
    SyncWithAPI,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    Auth(ActionAuth),
    Addons(ActionAddons),
    Settings(ActionSettings),
    Library(ActionLibrary),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLoad {
    Ctx,
    AddonDetails(AddonDetailsSelected),
    CatalogFiltered(CatalogFilteredSelected),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    LibraryFiltered(LibraryFilteredSelected),
    MetaDetails(MetaDetailsSelected),
    Player(PlayerSelected),
    Notifications,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    TimeChanged { time: u64, duration: u64 },
    Ended,
}

//
// Those messages are meant to be dispatched only by the users of the stremio-core crate and handled by the stremio-core crate
//
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Ctx(ActionCtx),
    Load(ActionLoad),
    Player(ActionPlayer),
    Unload,
}
