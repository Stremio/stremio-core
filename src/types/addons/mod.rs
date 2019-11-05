use serde_derive::*;
mod manifest;
mod resource_ref;
pub use self::manifest::*;
pub use self::resource_ref::*;
use crate::types::{MetaDetail, MetaPreview, Stream, SubtitlesSource};
mod manifest_tests;
use derive_more::*;

pub type TransportUrl = String;

#[derive(Default, PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorFlags {
    #[serde(default)]
    pub official: bool,
    #[serde(default)]
    pub protected: bool,
    #[serde(flatten)]
    #[serde(default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorPreview {
    pub manifest: ManifestPreview,
    pub transport_url: TransportUrl,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub manifest: Manifest,
    pub transport_url: TransportUrl,
    #[serde(default)]
    pub flags: DescriptorFlags,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRequest {
    pub base: TransportUrl,
    pub path: ResourceRef,
}
impl ResourceRequest {
    pub fn new(base: &str, path: ResourceRef) -> Self {
        let base = base.to_owned();
        ResourceRequest { base, path }
    }
    pub fn eq_no_extra(&self, other: &ResourceRequest) -> bool {
        self.base == other.base && self.path.eq_no_extra(&other.path)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, TryInto)]
#[serde(untagged)]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaPreview>,
    },
    #[serde(rename_all = "camelCase")]
    MetasDetailed {
        metas_detailed: Vec<MetaDetail>,
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
    Subtitles {
        subtitles: Vec<SubtitlesSource>,
    },
    Addons {
        addons: Vec<DescriptorPreview>,
    },
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
                        addon
                            .manifest
                            .catalogs
                            .iter()
                            .filter(|cat| cat.is_extra_supported(&extra))
                            .map(move |cat| {
                                (
                                    addon,
                                    ResourceRequest::new(
                                        &addon.transport_url,
                                        ResourceRef::with_extra(
                                            "catalog",
                                            &cat.type_name,
                                            &cat.id,
                                            extra,
                                        ),
                                    ),
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
                            ResourceRequest::new(&addon.transport_url, path.to_owned()),
                        )
                    })
                    .collect()
            }
        }
    }
}
