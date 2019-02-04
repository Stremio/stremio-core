use crate::state_types::catalogs::*;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
// @TODO use named fields for some variants
pub enum Action {
    Load(ActionLoad),
    // @TODO this should be renamed to LoadWithUser; we should also have UserLoaded and UserValue,
    // both having (user, addons)
    LoadWithAddons(Vec<Descriptor>, ActionLoad),

    NewState(usize),

    // @TODO those are temporary events, remove them
    CatalogRequested(RequestId),
    CatalogReceived(RequestId, Result<CatalogResponse, String>),
}

impl Action {
    pub fn addon_aggr_req(&self) -> Option<AggrRequest> {
        // @TODO map WithAddons(addons, CatalogGrouped()) to AllCatalogs
        // map WithAddons(addons, CatalogFiltered()) to FromAddon
        // etc.
        if let Action::LoadWithAddons(_addons, _action_load) = &self {
            // @TODO
            Some(AggrRequest::AllCatalogs { extra: "".into() })
        } else {
            None
        }
    }
}
