use serde_derive::*;

mod manifest;
pub use self::manifest::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: String,
    // @TODO flags
}

// @TODO should we replace resource with an enum?

// @TODO: another type for extra
pub type Extra = String;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Extra,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceRequest {
    pub transport_url: String,
    pub resource_ref: ResourceRef,
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AggrRequest {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs { extra: Extra },
    AllOfResource(ResourceRef),
    // @TODO this can be replaced by Request
    FromAddon(ResourceRequest),
}

impl AggrRequest {
    pub fn plan(&self, addons: &[Descriptor]) -> Vec<ResourceRequest> {
        // @TODO
        match &self {
            AggrRequest::AllCatalogs { extra: _ } => {
                // @TODO for each addon, go through each catalog, then filter by
                // is_supported_catalog
                vec![]
            }
            AggrRequest::AllOfResource(resource_ref) => {
                // filter all addons that match the resource_ref
                addons
                    .iter()
                    .filter(|addon| addon.manifest.is_supported(&resource_ref))
                    .map(|addon| ResourceRequest {
                        transport_url: addon.transport_url.to_owned(),
                        resource_ref: resource_ref.to_owned(),
                    })
                    .collect()
            }
            AggrRequest::FromAddon(req) => vec![req.to_owned()],
        }
    }
}
