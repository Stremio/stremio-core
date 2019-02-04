use crate::state_types::catalogs::*;
use crate::types::*;
use serde_derive::*;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Loadable {
    Catalog,
}

// @TODO AddonAggrRequest
// @TODO move this out of here; then we may be able to get rid of the prefixes
// @TODO a way to "plan" addon requests by combining addons and AddonAggrRequest
// perhaps impl AddonAddrRequest { pub fn plan() }
// then the plan might return Vec<AddonReqPlan> that we might Hash to match against the response
// later
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
pub enum AddonAggrRequest {
    AllOfResource(AddonResourceRef),
    FromAddon{ transport_url: String, resource_ref: AddonResourceRef },
    AllCatalogs{ extra: AddonExtra },
}

impl AddonAggrRequest {
    pub fn plan(_addons: &Vec<AddonDescriptor>) {
        // @TODO
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
    pub fn get_addon_request(&self) -> AddonAggrRequest {
        AddonAggrRequest::AllCatalogs{ extra: "".into() }
    }
}

// @TODO AddonCollection, Eq on AddonCollection
// Middleware actions: AddonRequest, AddonResponse
