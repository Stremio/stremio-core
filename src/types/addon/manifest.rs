use crate::types::addon::{ExtraValue, ResourcePath};
use derivative::Derivative;
use derive_more::Deref;
use either::Either;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Derivative, Debug))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    #[cfg_attr(test, derivative(Default(value = "Version::new(0, 0, 1)")))]
    pub version: Version,
    pub name: String,
    pub contact_email: Option<String>,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub background: Option<String>,
    pub types: Vec<String>,
    pub resources: Vec<ManifestResource>,
    pub id_prefixes: Option<Vec<String>>,
    #[serde(default)]
    pub catalogs: Vec<ManifestCatalog>,
    #[serde(default)]
    pub addon_catalogs: Vec<ManifestCatalog>,
    #[serde(default)]
    pub behavior_hints:ManifestBehaviorHints,
}

impl Manifest {
    pub fn is_resource_supported(&self, path: &ResourcePath) -> bool {
        match path.resource.as_str() {
            "catalog" => self.catalogs.iter().any(|catalog| {
                catalog.r#type == path.r#type
                    && catalog.id == path.id
                    && catalog.is_extra_supported(&path.extra)
            }),
            "addon_catalog" => self.addon_catalogs.iter().any(|catalog| {
                catalog.r#type == path.r#type
                    && catalog.id == path.id
                    && catalog.is_extra_supported(&path.extra)
            }),
            _ => {
                let resource = match self
                    .resources
                    .iter()
                    .find(|resource| resource.name() == path.resource)
                {
                    Some(resource) => resource,
                    None => return false,
                };
                let types = match resource {
                    ManifestResource::Short(_) => Some(&self.types),
                    ManifestResource::Full { types, .. } => types.as_ref(),
                };
                let id_prefixes = match resource {
                    ManifestResource::Short(_) => self.id_prefixes.as_ref(),
                    ManifestResource::Full { id_prefixes, .. } => id_prefixes.as_ref(),
                };
                let type_supported = types.map_or(false, |types| types.contains(&path.r#type));
                let id_supported = id_prefixes.map_or(true, |id_prefixes| {
                    id_prefixes.iter().any(|prefix| path.id.starts_with(prefix))
                });
                type_supported && id_supported
            }
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Derivative, Debug))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
pub struct ManifestPreview {
    pub id: String,
    #[cfg_attr(test, derivative(Default(value = "Version::new(0, 0, 1)")))]
    pub version: Version,
    pub name: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub background: Option<String>,
    pub types: Vec<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum ManifestResource {
    Short(String),
    #[serde(rename_all = "camelCase")]
    Full {
        name: String,
        types: Option<Vec<String>>,
        id_prefixes: Option<Vec<String>>,
    },
}

impl ManifestResource {
    #[inline]
    fn name(&self) -> &str {
        match self {
            ManifestResource::Short(name) => name,
            ManifestResource::Full { name, .. } => name,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ManifestCatalog {
    pub r#type: String,
    pub id: String,
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: ManifestExtra,
}

impl ManifestCatalog {
    pub fn is_extra_supported(&self, extra: &[ExtraValue]) -> bool {
        let all_supported = extra.iter().all(|extra_value| {
            self.extra
                .iter()
                .any(|extra_prop| extra_prop.name == extra_value.name)
        });
        let required_satisfied = self
            .extra
            .iter()
            .filter(|extra_prop| extra_prop.is_required)
            .all(|extra_prop| {
                extra
                    .iter()
                    .any(|extra_value| extra_value.name == extra_prop.name)
            });
        all_supported && required_satisfied
    }
    pub fn default_required_extra(&self) -> Option<Vec<ExtraValue>> {
        self.extra
            .iter()
            .filter(|extra| extra.is_required)
            .map(|extra| {
                extra
                    .options
                    .as_ref()
                    .and_then(|options| options.first())
                    .map(|first_option| ExtraValue {
                        name: extra.name.to_owned(),
                        value: first_option.to_owned(),
                    })
            })
            .collect::<Option<Vec<_>>>()
    }
}

#[derive(Derivative, Clone, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum ManifestExtra {
    #[derivative(Default)]
    Full {
        #[serde(rename = "extra")]
        props: Vec<ExtraProp>,
    },
    Short {
        #[serde(default, rename = "extraRequired")]
        required: Vec<String>,
        #[serde(default, rename = "extraSupported")]
        supported: Vec<String>,
    },
}

impl ManifestExtra {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Cow<ExtraProp>> + 'a {
        match &self {
            ManifestExtra::Full { props } => Either::Left(props.iter().map(Cow::Borrowed)),
            ManifestExtra::Short {
                required,
                supported,
            } => Either::Right(supported.iter().map(move |name| {
                Cow::Owned(ExtraProp {
                    name: name.to_owned(),
                    is_required: required.contains(name),
                    ..Default::default()
                })
            })),
        }
    }
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ExtraProp {
    pub name: String,
    #[serde(default)]
    pub is_required: bool,
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub options_limit: OptionsLimit,
}

#[derive(Clone, Deref, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct OptionsLimit(pub usize);

impl Default for OptionsLimit {
    fn default() -> OptionsLimit {
        OptionsLimit(1)
    }
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ManifestBehaviorHints {
    pub adult: bool,
    pub p2p: bool,
    pub configurable: bool,
    pub configuration_required: bool,
}
