use crate::state_types::catalogs::*;
use crate::types::*;
use serde_derive::*;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Loadable {
    Catalog,
}


// @TODO move all of this out of here


// @TODO: another type for extra
pub type AddonExtra = String;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddonResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: AddonExtra,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddonRequest {
    pub transport_url: String,
    pub resource_ref: AddonResourceRef,
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AddonAggrRequest {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs{ extra: AddonExtra },
    AllOfResource(AddonResourceRef),
    // @TODO this can be replaced by AddonRequest
    FromAddon(AddonRequest),
}

impl AddonAggrRequest {
    pub fn plan(&self, addons: &Vec<AddonDescriptor>) -> Vec<AddonRequest> {
        // @TODO
        match &self {
            AddonAggrRequest::AllCatalogs{ extra } => {
                // @TODO for each addon, go through each catalog, then filter by
                // is_supported_catalog 
                vec![]
            },
            AddonAggrRequest::AllOfResource(resource_ref) => {
                // filter all addons that match the resource_ref
                addons.iter()
                    .filter(|addon| addon.manifest.is_supported(&resource_ref.resource, &resource_ref.type_name, &resource_ref.id))
                    .map(|addon| AddonRequest{ transport_url: addon.transport_url.to_owned(), resource_ref: resource_ref.to_owned() })
                    .collect()
            },
            AddonAggrRequest::FromAddon(req) => {
                vec![req.to_owned()]
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
// @TODO use named fields for some variants
pub enum Action {
    Init,
    Load(Loadable),
    // @TODO those are temporary events, remove them
    CatalogRequested(RequestId),
    CatalogReceived(RequestId, Result<CatalogResponse, String>),
    AddonsLoaded(Vec<AddonDescriptor>),
    WithAddons(Vec<AddonDescriptor>, Box<Action>),
    NewState(usize),
}

impl Action {
    pub fn get_addon_request(&self) -> Option<AddonAggrRequest> {
        Some(AddonAggrRequest::AllCatalogs{ extra: "".into() })
    }
}

// @TODO AddonCollection, Eq on AddonCollection
// Middleware actions: AddonRequest, AddonResponse
