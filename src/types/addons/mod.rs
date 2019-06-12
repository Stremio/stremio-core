use serde_derive::*;
use std::convert::TryInto;
mod manifest;
mod resource_ref;
pub use self::manifest::*;
pub use self::resource_ref::*;
use crate::types::{MetaDetail, MetaPreview, Stream};
mod manifest_tests;

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
    pub base: TransportUrl,
    pub path: ResourceRef,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaPreview>,
    },
    Meta {
        // NOTE: we are not putting this in Option<>, since that way it gives us a valid
        // fallback to all of the previous variants, therefore resulting in inaccurate err messages
        // To support other /meta/ responses (meta extensions), we should make a MetaExt variant
        meta: MetaDetail,
    },
    Streams {
        streams: Vec<Stream>,
    },
    // @TODO subtitles
}
impl TryInto<Vec<MetaPreview>> for ResourceResponse {
    type Error = ();
    fn try_into(self) -> Result<Vec<MetaPreview>, ()> {
        match self {
            ResourceResponse::Metas { metas } => Ok(metas),
            _ => Err(()),
        }
    }
}
impl TryInto<Vec<Stream>> for ResourceResponse {
    type Error = ();
    fn try_into(self) -> Result<Vec<Stream>, ()> {
        match self {
            ResourceResponse::Streams { streams } => Ok(streams),
            _ => Err(()),
        }
    }
}

// This is going from the most general to the most concrete aggregation request
#[derive(Debug, Clone)]
pub enum AggrRequest<'a> {
    // @TODO should AllCatalogs have optional resource and type_name?
    AllCatalogs { extra: &'a Vec<ExtraProp> },
    AllOfResource(ResourceRef),
}

// Given an AggrRequest, which describes how to request data from *all* addons,
// return a vector of individual addon requests
impl AggrRequest<'_> {
    pub fn plan<'a>(&self, addons: &'a [Descriptor]) -> Vec<(&'a Descriptor, ResourceRequest)> {
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
                            .map(move |cat| {
                                (
                                    addon,
                                    ResourceRequest {
                                        base: transport_url.to_owned(),
                                        path: ResourceRef::with_extra(
                                            "catalog",
                                            &cat.type_name,
                                            &cat.id,
                                            extra,
                                        ),
                                    },
                                )
                            })
                    })
                    .flatten()
                    .collect()
            }
            AggrRequest::AllOfResource(path) => {
                // filter all addons that match the path
                addons
                    .iter()
                    .filter(|addon| addon.manifest.is_supported(&path))
                    .map(|addon| {
                        (
                            addon,
                            ResourceRequest {
                                base: addon.transport_url.to_owned(),
                                path: path.to_owned(),
                            },
                        )
                    })
                    .collect()
            }
        }
    }
}
