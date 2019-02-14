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
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login { username: String, password: String },
    Signup { username: String, password: String },
    Logout,
    PullAddons,
    PushAddons,
    // @TODO maybe PullUser, PushUser?
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    // Input actions
    Load(ActionLoad),

    // user-specific action
    UserOp(ActionUser),

    // Intermediery
    // @TODO this should be renamed to LoadWithUser; we should also have UserLoaded and UserValue,
    // both having (user, addons)
    LoadWithAddons(Vec<Descriptor>, ActionLoad),
    AddonResponse(ResourceRequest, Result<ResourceResponse, String>),

    // Output actions
    // @TODO consider better error reporting for UserMiddlewareError
    // there are non-fatal and fatal errors, and 3-4 classes of errors (network, deserialization,
    // storage, etc.)
    // @TODO consider the name UserMError
    UserMiddlewareError(String),
    NewState(usize),
}
