use crate::state_types::catalogs::*;
use crate::types::*;
use serde_derive::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
// @TODO some generic way to do actions; perhaps enums should be avoided
// or alternatively we'd use a lot of From and Into in order to have separate events for the
// middlwares
// @TODO use named fields for some variants
pub enum Action {
    Init,
    Open,
    // @TODO those are temporary events, remove them
    LoadCatalogs,
    CatalogRequested(RequestId),
    CatalogReceived(RequestId, Result<CatalogResponse, String>),
    AddonsLoaded(Vec<AddonDescriptor>),
    WithAddons(Vec<AddonDescriptor>, Box<Action>),
    NewState(usize),
}

// @TODO AddonCollection, Eq on AddonCollection
// Middleware actions: AddonRequest, AddonResponse
