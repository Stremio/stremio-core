use crate::state_types::models::catalogs_with_extra::Selected as CatalogsWithExtraSelected;
use crate::state_types::models::ctx::Settings;
use crate::types::addons::{Descriptor, ResourceRequest, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::{LibItem, MetaPreview, Stream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionUser {
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
    Add(Box<MetaPreview>),
    Remove { id: String },
    Update(Box<LibItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionCtx {
    SyncWithAPI,
    RetrieveFromStorage,
    PersistToStorage,
    User(ActionUser),
    Addons(ActionAddons),
    Settings(ActionSettings),
    Library(ActionLibrary),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLoad {
    Notifications,
    CatalogsWithExtra(CatalogsWithExtraSelected),
    CatalogFiltered(ResourceRequest),
    LibraryFiltered {
        type_name: String,
        sort_prop: Option<String>,
    },
    MetaDetails {
        type_name: String,
        id: String,
        video_id: Option<String>,
    },
    AddonDetails(TransportUrl),
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
