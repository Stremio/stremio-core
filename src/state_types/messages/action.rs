use crate::state_types::models::ctx::Settings;
use crate::state_types::models::StreamingServerSettings;
use crate::types::addons::{Descriptor, ExtraProp, ResourceRequest, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::{LibItem, MetaPreview, Stream};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLoad {
    UserData,
    CatalogsWithExtra {
        extra: Vec<ExtraProp>,
    },
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
    AddonDetails {
        transport_url: String,
    },
    Notifications,
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
    // TODO consider PullUser, PushUser?
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionAddon {
    Install(Descriptor),
    Uninstall { transport_url: TransportUrl },
    PullAndUpdateAddons,
    PushAddons,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionSettings {
    LoadStreamingServer,
    StoreStreamingServer(Box<StreamingServerSettings>),
    UpdateSettings(Settings),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionLibrary {
    LibSync,
    LibUpdate(LibItem),
    AddToLibrary {
        meta_item: MetaPreview,
        now: DateTime<Utc>,
    },
    RemoveFromLibrary {
        id: String,
        now: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum ActionPlayer {
    TimeChanged { time: u64, duration: u64 },
    Ended,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Load(ActionLoad),
    User(ActionUser),
    Addon(ActionAddon),
    Settings(ActionSettings),
    Library(ActionLibrary),
    Player(ActionPlayer),
    Unload,
}
