use crate::types::*;
use serde_derive::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ActionLoad {
    // @TODO most of these values need content
    CatalogGrouped,
    CatalogFiltered,
    Detail,
    Streams,
    AddonCatalog,
}
impl ActionLoad {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        // @TODO map CatalogGrouped to AllCatalogs
        // map CatalogFiltered  to FromAddon
        // etc.
        Some(AggrRequest::AllCatalogs { extra: "".into() })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Action {
    Load(ActionLoad),
    // @TODO this should be renamed to LoadWithUser; we should also have UserLoaded and UserValue,
    // both having (user, addons)
    LoadWithAddons(Vec<Descriptor>, ActionLoad),

    NewState(usize),

    AddonResponse(ResourceRequest, Result<CatalogResponse, String>),
}
