use crate::types::addons::*;
use crate::types::LibItem;
use serde_derive::*;

//
// Input actions: those are triggered by users
//
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogGrouped { extra: Vec<ExtraProp> },
    CatalogFiltered(ResourceRequest),
    MetaDetail { type_name: String, id: String, video_id: Option<String> },
    Notifications,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login { email: String, password: String },
    Register { email: String, password: String },
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
    AddonOp(ActionAddon),
    UserOp(ActionUser),
}
