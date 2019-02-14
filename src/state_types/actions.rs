use crate::types::*;
use serde_derive::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    // @TODO most of these values need content
    CatalogGrouped { extra: Extra },
    CatalogFiltered,
    Detail,
    Streams,
    AddonCatalog,
}
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        // @TODO map CatalogFilteredto FromAddon
        // @TODO map Detail/Streams to AllOfResource
        // etc.
        match self {
            ActionLoad::CatalogGrouped { extra } => Some(AggrRequest::AllCatalogs {
                extra: extra.to_owned(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "load", content = "args")]
pub enum ActionUser {
    Login{ username: String, password: String },
    Signup{ username: String, password: String },
    Logout,
    PullAddons,
    PushAddons,
    // @TODO maybe PullUser, PushUser?
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    Load(ActionLoad),

    // user-specific action
    User(ActionUser),
    // @TODO this should be renamed to LoadWithUser; we should also have UserLoaded and UserValue,
    // both having (user, addons)
    LoadWithAddons(Vec<Descriptor>, ActionLoad),

    NewState(usize),

    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),
}
