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
    CatalogFiltered { resource_req: ResourceRequest },
    Detail { type_name: String, id: String },
    Streams { type_name: String, id: String },
    // @TODO most of these values need content
    AddonCatalog,
}
// @TODO: drop this from here
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        match self {
            ActionLoad::CatalogGrouped { extra } => Some(AggrRequest::AllCatalogs {
                extra: extra.to_owned(),
            }),
            ActionLoad::CatalogFiltered { resource_req } => {
                Some(AggrRequest::FromAddon(resource_req.to_owned()))
            }
            ActionLoad::Detail { type_name, id } => Some(AggrRequest::AllOfResource(
                ResourceRef::without_extra("meta", type_name, id),
            )),
            ActionLoad::Streams { type_name, id } => Some(AggrRequest::AllOfResource(
                ResourceRef::without_extra("stream", type_name, id),
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login { email: String, password: String },
    Register { email: String, password: String },
    Logout,
    PullAddons,
    PushAddons,
    // @TODO consider PullUser, PushUser?
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    LoadCtx,
    Load(ActionLoad),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
    LibSync,
    LibUpdate(LibItem),
}
