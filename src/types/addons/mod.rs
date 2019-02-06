use serde_derive::*;

mod manifest;
pub use self::manifest::*;
use crate::types::meta_item::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: String,
    // @TODO flags
}

// @TODO should we replace resource with an enum?

// @TODO: a different type for extra (HashMap?)
pub type Extra = String;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Extra,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRequest {
    pub transport_url: String,
    pub resource_ref: ResourceRef,
}

// If we ever need to return two properties (e.g. `{streams, ads}`) we can use structs
// and the Untagged representation: https://serde.rs/enum-representations.html#untagged
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResourceResponse {
    Metas(Vec<MetaPreview>),
    Meta(MetaItem),
    //Streams(Vec<Stream>),
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Clone)]
pub enum AggrRequest {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs { extra: Extra },
    AllOfResource(ResourceRef),
    FromAddon(ResourceRequest),
}

impl AggrRequest {
    pub fn plan(&self, addons: &[Descriptor]) -> Vec<ResourceRequest> {
        match &self {
            AggrRequest::AllCatalogs { extra } => {
                // @TODO is_supported_catalog
                addons
                    .iter()
                    .map(|addon| {
                        let transport_url = addon.transport_url.to_owned();
                        addon
                            .manifest
                            .catalogs
                            .iter()
                            .filter(|cat| cat.extra_required.is_empty())
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
