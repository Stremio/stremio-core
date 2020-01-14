use crate::state_types::models::addon_details::Selected as AddonDetailsSelected;
use crate::state_types::models::catalog_filtered::Selected as CatalogFilteredSelected;
use crate::state_types::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::state_types::models::ctx::Settings;
use crate::types::addons::{Descriptor, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::{LibItem, MetaPreview, Stream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionAddons {
    PushToAPI,
    PullFromAPI,
    Install(Box<Descriptor>),
    Uninstall { transport_url: TransportUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionSettings {
    Update(Box<Settings>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLibrary {
    SyncWithAPI,
    Add(Box<MetaPreview>),
    Remove { id: String },
    Update(Box<LibItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    RetrieveFromStorage,
    Auth(ActionAuth),
    Addons(ActionAddons),
    Settings(ActionSettings),
    Library(ActionLibrary),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLoad {
    AddonDetails(AddonDetailsSelected),
    CatalogFiltered(CatalogFilteredSelected),
    CatalogsWithExtra(CatalogsWithExtraSelected),
    Notifications,
    LibraryFiltered {
        type_name: String,
        sort_prop: Option<String>,
    },
    MetaDetails {
        type_name: String,
        id: String,
        video_id: Option<String>,
    },
    Player {
        transport_url: String,
        type_name: String,
        id: String,
        video_id: String,
        stream: Stream,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    TimeChanged { time: u64, duration: u64 },
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Ctx(ActionCtx),
    Load(ActionLoad),
    Player(ActionPlayer),
    Unload,
}
