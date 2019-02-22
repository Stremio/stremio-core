use serde_derive::*;
mod manifest;
mod resource_ref;
pub use self::manifest::*;
pub use self::resource_ref::*;
use crate::types::meta_item::*;

pub type TransportUrl = String;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: TransportUrl,
    // @TODO flags
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRequest {
    pub transport_url: TransportUrl,
    pub resource_ref: ResourceRef,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaPreview>,
        #[serde(default)]
        skip: u32,
        #[serde(default)]
        has_more: bool,
    },
    Meta {
        // NOTE: we are not putting this in Option<>, since that way it gives us a valid
        // fallback to all of the previous variants, therefore resulting in inaccurate err messages
        // To support other /meta/ responses (meta extensions), we should make a MetaExt variant
        meta: MetaItem,
    },
    //Streams { streams: Vec<Stream> },
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Clone)]
pub enum AggrRequest {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs { extra: Vec<ExtraProp> },
    AllOfResource(ResourceRef),
    FromAddon(ResourceRequest),
}

// Given an AggrRequest, which describes how to request data from *all* addons,
// return a vector of individual addon requests
impl AggrRequest {
    pub fn plan(&self, addons: &[Descriptor]) -> Vec<ResourceRequest> {
        match &self {
            AggrRequest::AllCatalogs { extra } => {
                // create a request for each catalog that matches the required extra properties
                addons
                    .iter()
                    .map(|addon| {
                        let transport_url = addon.transport_url.to_owned();
                        addon
                            .manifest
                            .catalogs
                            .iter()
                            .filter(|cat| cat.is_extra_supported(&extra))
                            .map(move |cat| ResourceRequest {
                                transport_url: transport_url.to_owned(),
                                resource_ref: ResourceRef {
                                    resource: "catalog".to_owned(),
                                    type_name: cat.type_name.to_owned(),
                                    id: cat.id.to_owned(),
                                    extra: extra.to_owned(),
                                },
                            })
                    })
                    .flatten()
                    .collect()
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
