use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    constants::CATALOG_RESOURCE_NAME,
    types::addon::{Descriptor, ExtraProp, ManifestResource},
};

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
///
/// The url paths look as follows:
/// - Without extra values: `{resource}/{type}/{id}.json`
/// - With extra values: `{resource}/{type}/{id}/{extra}.json`
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct ResourcePath {
    /// The resource we want to fetch from the addon.
    ///
    /// # Examples
    ///
    /// - [`CATALOG_RESOURCE_NAME`]
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
    /// they will be encoded using [`query_params_encode()`](crate::types::query_params_encode()).
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
pub enum ExtraType {
    /// the extra supports a list of ids
    Ids {
        /// The extra name.
        /// It will be checked against the addon manifest to validate it's supported
        extra_name: String,
        /// Ids must be ordered if we want to correctly limit the request ids,
        /// based on the ExtraValue OptionsLimit supported by the addon.
        ///
        /// The first value is the id of the item while the second is an optional type
        id_types: Vec<(String, String)>,
        /// A set limit on the requested ids per addon.
        /// The smaller value of the two will be taken: defined limit or the ExtraValues OptionsLimit.
        limit: Option<usize>,
    },
}

#[derive(Clone, Debug)]
pub enum AggrRequest<'a> {
    AllCatalogs {
        extra: &'a Vec<ExtraValue>,
        r#type: &'a Option<String>,
    },
    CatalogsFiltered(Vec<ExtraType>),
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
                                        CATALOG_RESOURCE_NAME,
                                        &catalog.r#type,
                                        &catalog.id,
                                        extra,
                                    ),
                                ),
                            )
                        })
                })
                .collect(),
            AggrRequest::CatalogsFiltered(extra_types) => {
                let extra_names = extra_types
                    .iter()
                    .map(|extra_type| match extra_type {
                        ExtraType::Ids { extra_name, .. } => extra_name.to_owned(),
                    })
                    .collect::<Vec<_>>();

                let addon_requests = extra_types
                    .iter()
                    .flat_map(|extra_type| match extra_type {
                        ExtraType::Ids {
                            extra_name,
                            id_types,
                            limit: requested_limit,
                        } => {
                            addons
                                .iter()
                                .flat_map(|addon| {
                                    addon
                                        .manifest
                                        .catalogs
                                        .iter()
                                        .filter(|catalog| {
                                            // check if all extras are supported
                                            catalog.are_extra_names_supported(&extra_names)
                                        })
                                        // handle the supported catalogs
                                        .filter_map(move |catalog| {
                                            let mut supported_ids =
                                                id_types.iter().filter_map(|(id, item_type)| {
                                                    // is `catalog` Resource supported and it's types and id prefixes (if applicable) respected?
                                                    let catalog_resource_supported = addon.manifest.resources.iter().any(|resource| {
                                                        match resource {
                                                            // if we have a short `catalog` resource, they we support it
                                                            ManifestResource::Short(name) if name == CATALOG_RESOURCE_NAME => true,
                                                            // if we have a log `catalog` resource, check if the id prefix and type are supported
                                                            ManifestResource::Full { name, types, id_prefixes } if name == CATALOG_RESOURCE_NAME => {
                                                                // do we have types?
                                                                // if we do - check if the type is included / supported
                                                                // if no types are listed - we default to no types supported!
                                                                let item_type_supported = types
                                                                    .as_ref()
                                                                    .map(|supported_types| supported_types.contains(item_type))
                                                                    .unwrap_or(true);
                                                                let id_supported = id_prefixes
                                                                    .as_ref()
                                                                    .map(|supported_prefixes| {
                                                                        // on empty prefixes we consider the id supported!
                                                                        // otherwise check in the list
                                                                        supported_prefixes.is_empty() || supported_prefixes.iter().any(|prefix| {
                                                                            id.starts_with(prefix)
                                                                        })
                                                                    })
                                                                    .unwrap_or(true);

                                                                item_type_supported && id_supported
                                                            },
                                                            // in any other case, for any other resource, we do not support this id
                                                            _ => false
                                                        }
                                                    });

                                                    // if catalog resource doesn't support the id or if the catalog is not the same type
                                                    if !catalog_resource_supported || &catalog.r#type != item_type {
                                                        // we do not support the id
                                                        return None;
                                                    }

                                                    // check if the addon supports the specified id in it's the global manifest id prefixes
                                                    let manifest_id_prefixes_supported = addon.manifest.id_prefixes.as_ref().map(|supported_prefixes| {
                                                        // on empty prefixes we consider the id supported!
                                                        // otherwise check in the list
                                                        supported_prefixes.is_empty() || supported_prefixes.iter().any(|prefix| {
                                                            id.starts_with(prefix)
                                                        })
                                                    }).unwrap_or(true);

                                                    let manifest_type_supported = addon
                                                        .manifest
                                                        .types
                                                        .contains(item_type);

                                                    if !manifest_id_prefixes_supported || !manifest_type_supported {
                                                        return None;
                                                    }

                                                    Some(id.clone())
                                                }).collect::<Vec<String>>();

                                            if supported_ids.is_empty() {
                                                return None;
                                            }

                                            // make sure we respect the addon specified OptionsLimit
                                            let extra_limit =
                                                catalog.extra.iter().find_map(|extra_prop| {
                                                    if &extra_prop.name == extra_name {
                                                        Some(extra_prop.options_limit)
                                                    } else {
                                                        None
                                                    }
                                                });

                                            let request_limit = match (extra_limit, requested_limit)
                                            {
                                                // take the smaller value for limiting the ids in the request, either:
                                                // - the options limit defined by the addon
                                                // - the limit passed to the request
                                                (Some(options_limit), Some(requested_limit)) => {
                                                    options_limit.0.min(*requested_limit)
                                                }
                                                (Some(options_limit), None) => options_limit.0,
                                                (None, Some(requested_limit)) => *requested_limit,
                                                // limited only by the size of the array
                                                (None, None) => usize::MAX,
                                            };

                                            // make sure we don't make an out-of-bound on the array
                                            let last_index = request_limit.min(supported_ids.len());
                                            let supported_ids_trimmed =
                                                match supported_ids.get_mut(..last_index) {
                                                    Some(ids) if !ids.is_empty() => {
                                                        // after we've filtered by recency
                                                        // we order the ids "alphabetically" for our needs (check `sort()` for more details)
                                                        // to improve caching in addons
                                                        ids.sort();

                                                        ids.join(",")
                                                    }
                                                    _ => return None,
                                                };
                                            // build the extra values
                                            let extra = &[ExtraValue {
                                                name: extra_name.to_owned(),
                                                value: supported_ids_trimmed,
                                            }];

                                            Some((
                                                addon,
                                                ResourceRequest::new(
                                                    addon.transport_url.to_owned(),
                                                    ResourcePath::with_extra(
                                                        CATALOG_RESOURCE_NAME,
                                                        &catalog.r#type,
                                                        &catalog.id,
                                                        extra,
                                                    ),
                                                ),
                                            ))
                                        })
                                })
                                .collect::<Vec<_>>()
                        }
                    })
                    .collect();

                addon_requests
            }
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
