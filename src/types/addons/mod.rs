use serde_derive::*;
use std::fmt;
use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

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

pub type Extra = Vec<(String, String)>;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Extra,
}
impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, DEFAULT_ENCODE_SET),
            &utf8_percent_encode(&self.type_name, DEFAULT_ENCODE_SET),
            &utf8_percent_encode(&self.id, DEFAULT_ENCODE_SET)
        )?;
        if !self.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for (k, v) in self.extra.iter() {
                extra_encoded.append_pair(&k, &v);
            }
            write!(f, "/{}", &extra_encoded.finish())?;
        }
        write!(f, ".json")
    }
}
// @TODO from String?

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRequest {
    pub transport_url: String,
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
        meta: Option<MetaItem>,
    },
    //Streams { streams: Vec<Stream> },
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Clone)]
pub enum AggrRequest {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs { extra: Extra },
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
