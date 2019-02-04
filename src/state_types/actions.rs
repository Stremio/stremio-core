use crate::state_types::catalogs::*;
use crate::types::*;
use serde_derive::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Loadable {
    Catalog,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
// @TODO use named fields for some variants
pub enum Action {
    Init,
    Load(Loadable),
    // @TODO those are temporary events, remove them
    CatalogRequested(RequestId),
    CatalogReceived(RequestId, Result<CatalogResponse, String>),
    AddonsLoaded(Vec<Descriptor>),
    WithAddons(Vec<Descriptor>, Box<Action>),
    NewState(usize),
}

impl Action {
    pub fn get_addon_request(&self) -> Option<AggrRequest> {
        // @TODO map WithAddons(addons, CatalogGrouped()) to AllCatalogs
        // map WithAddons(addons, CatalogFiltered()) to FromAddon
        // etc.
        Some(AggrRequest::AllCatalogs { extra: "".into() })
    }
}

// @TODO AddonCollection, Eq on AddonCollection
// Middleware actions: AddonRequest, AddonResponse
