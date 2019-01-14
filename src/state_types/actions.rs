use crate::types::*;
use serde_derive::*;
use crate::state_types::catalogs::*;
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
// @TODO some generic way to do actions; perhaps enums should be avoided
// or alternatively we'd use a lot of From and Into in order to have separate events for the
// middlwares
pub enum Action {
    // @TODO args
    Init,
    Open,
    // @TODO those are temporary events, remove them
    LoadCatalogs,
    CatalogRequested(RequestId),
    CatalogReceived(RequestId, Result<CatalogResponse, String>),
    CatalogGroupedNew(Box<CatalogGrouped>),
    AddonsLoaded(Box<Vec<AddonDescriptor>>),
    WithAddons(Box<Vec<AddonDescriptor>>, Box<Action>),
}

// @TODO AddonCollection, Eq on AddonCollection
// Middleware actions: AddonRequest, AddonResponse


