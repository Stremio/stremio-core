use crate::state_types::{Settings, StreamingServerSettings};
use crate::types::addons::{Descriptor, ExtraProp, ResourceRequest, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::LibItem;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogGrouped {
        extra: Vec<ExtraProp>,
    },
    CatalogFiltered(ResourceRequest),
    MetaDetails {
        type_name: String,
        id: String,
        video_id: Option<String>,
    },
    Notifications,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "settings", content = "args")]
pub enum ActionSettings {
    // Although we load the streaming server settings together with the context
    // there is also a way to reload it separately in cases this is necessary.
    LoadStreamingServer,
    StoreStreamingServer(Box<StreamingServerSettings>),
    Store(Box<Settings>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "userOp", content = "args")]
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
    PullAndUpdateAddons,
    PushAddons,
    LibSync,
    LibUpdate(LibItem),
    // @TODO consider PullUser, PushUser?
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    LoadCtx,
    Load(ActionLoad),
    Settings(ActionSettings),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
}
