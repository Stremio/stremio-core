use crate::state_types::{Settings, StreamingServerSettings};
use crate::types::addons::*;
use crate::types::api::GDPRConsent;
use crate::types::LibItem;
use serde_derive::*;

//
// Input actions: those are triggered by users
//
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogGrouped {
        extra: Vec<ExtraProp>,
    },
    CatalogFiltered(ResourceRequest),
    Detail {
        type_name: String,
        id: String,
        video_id: Option<String>,
    },
    Streams {
        type_name: String,
        id: String,
    },
    Notifications,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "settings", content = "args")]
pub enum ActionSettings {
    // Although we load the streaming server settings together with the context
    // there is also a way to reload it separately in cases this is necessary.
    LoadStreamingServer,
    StoreStreamingServer(Box<StreamingServerSettings>),
    Store(Box<Settings>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    LoadCtx,
    Load(ActionLoad),
    Settings(ActionSettings),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
}
