use crate::types::addon::{Descriptor, ExtraProp};
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, From, Into, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(from = "(String, String)", into = "(String, String)")]
pub struct ExtraValue {
    pub name: String,
    pub value: String,
}

pub trait ExtraExt {
    fn remove_all(self, prop: &ExtraProp) -> Self;
    fn extend_one(self, prop: &ExtraProp, value: Option<String>) -> Self;
}

impl ExtraExt for Vec<ExtraValue> {
    fn remove_all(self, prop: &ExtraProp) -> Self {
        self.into_iter().filter(|ev| ev.name != prop.name).collect()
    }
    fn extend_one(self, prop: &ExtraProp, value: Option<String>) -> Self {
        let (extra, other_extra) = self
            .into_iter()
            .partition::<Vec<ExtraValue>, _>(|ev| ev.name == prop.name);
        let extra = match value {
            Some(value) if *prop.options_limit == 1 => vec![ExtraValue {
                name: prop.name.to_owned(),
                value,
            }],
            Some(value) if *prop.options_limit > 1 => {
                if extra.iter().any(|ev| ev.value == value) {
                    extra.into_iter().filter(|ev| ev.value != value).collect()
                } else {
                    vec![ExtraValue {
                        name: prop.name.to_owned(),
                        value,
                    }]
                    .into_iter()
                    .chain(extra)
                    .take(*prop.options_limit)
                    .collect()
                }
            }
            None if !prop.is_required => vec![],
            _ if *prop.options_limit == 0 => vec![],
            _ => extra,
        };
        extra.into_iter().chain(other_extra).collect()
    }
}

/// The full resource path, query, etc. for Addon requests
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct ResourcePath {
    /// The resource we want to fetch from the addon.
    ///
    /// # Examples
    ///
    /// - [`CATALOG_RESOURCE_NAME`](crate::constants::CATALOG_RESOURCE_NAME)
    /// - [`META_RESOURCE_NAME`](crate::constants::META_RESOURCE_NAME)
    /// - [`SUBTITLES_RESOURCE_NAME`](crate::constants::SUBTITLES_RESOURCE_NAME)
    /// - [`STREAM_RESOURCE_NAME`](crate::constants::STREAM_RESOURCE_NAME)
    pub resource: String,
    /// # Examples
    ///
    /// - `series`
    /// - `channel`
    pub r#type: String,
    /// The id of the endpoint that we want to request.
    /// This could, for example:
    /// - `last-videos/{extra}.json`
    /// - `tt7440726` (`meta/series/tt7440726.json`)
    pub id: String,
    /// Extra query parameters to be passed to the endpoint
    ///
    /// When calling the endpoint using the [`AddonHTTPTransport`](crate::addon_transport::AddonHTTPTransport),
    /// they will be encoded using [`query_params_encode`](crate::types::query_params_encode).
    pub extra: Vec<ExtraValue>,
}

impl ResourcePath {
    #[inline]
    pub fn without_extra(resource: &str, r#type: &str, id: &str) -> Self {
        ResourcePath {
            resource: resource.to_owned(),
            r#type: r#type.to_owned(),
            id: id.to_owned(),
            extra: vec![],
        }
    }
    #[inline]
    pub fn with_extra(resource: &str, r#type: &str, id: &str, extra: &[ExtraValue]) -> Self {
        ResourcePath {
            resource: resource.to_owned(),
            r#type: r#type.to_owned(),
            id: id.to_owned(),
            extra: extra.to_owned(),
        }
    }
    #[inline]
    pub fn get_extra_first_value(&self, name: &str) -> Option<&String> {
        self.extra
            .iter()
            .find(|extra_value| extra_value.name == name)
            .map(|extra_value| &extra_value.value)
    }
    #[inline]
    pub fn eq_no_extra(&self, other: &ResourcePath) -> bool {
        self.resource == other.resource && self.r#type == other.r#type && self.id == other.id
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ResourceRequest {
    pub base: Url,
    pub path: ResourcePath,
}

impl ResourceRequest {
    pub fn new(base: Url, path: ResourcePath) -> Self {
        ResourceRequest { base, path }
    }
    #[inline]
    pub fn eq_no_extra(&self, other: &ResourceRequest) -> bool {
        self.base == other.base && self.path.eq_no_extra(&other.path)
    }
}

#[derive(Clone, Debug)]
pub enum AggrRequest<'a> {
    AllCatalogs {
        extra: &'a Vec<ExtraValue>,
        r#type: &'a Option<String>,
    },
    AllOfResource(ResourcePath),
}

impl AggrRequest<'_> {
    pub fn plan<'a>(&self, addons: &'a [Descriptor]) -> Vec<(&'a Descriptor, ResourceRequest)> {
        match &self {
            AggrRequest::AllCatalogs { extra, r#type } => addons
                .iter()
                .flat_map(|addon| {
                    addon
                        .manifest
                        .catalogs
                        .iter()
                        .filter(|catalog| {
                            catalog.is_extra_supported(extra)
                                && r#type
                                    .as_ref()
                                    .map(|r#type| catalog.r#type == *r#type)
                                    .unwrap_or(true)
                        })
                        .map(move |catalog| {
                            (
                                addon,
                                ResourceRequest::new(
                                    addon.transport_url.to_owned(),
                                    ResourcePath::with_extra(
                                        "catalog",
                                        &catalog.r#type,
                                        &catalog.id,
                                        extra,
                                    ),
                                ),
                            )
                        })
                })
                .collect(),
            AggrRequest::AllOfResource(path) => addons
                .iter()
                .filter(|addon| addon.manifest.is_resource_supported(path))
                .map(|addon| {
                    (
                        addon,
                        ResourceRequest::new(addon.transport_url.to_owned(), path.to_owned()),
                    )
                })
                .collect(),
        }
    }
}
