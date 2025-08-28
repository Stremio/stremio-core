use std::borrow::Cow;

use derivative::Derivative;
use derive_more::Deref;
use either::Either;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DefaultOnError, DefaultOnNull, DeserializeAs, NoneAsEmptyString};
use url::Url;

use crate::constants::SKIP_EXTRA_PROP;
use crate::types::addon::{ExtraValue, ResourcePath};
use crate::types::{UniqueVec, UniqueVecAdapter};

/// Re-export the semver::Version
pub use semver::Version;

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    #[cfg_attr(test, derivative(Default(value = "Version::new(0, 0, 1)")))]
    pub version: Version,
    pub name: String,
    pub contact_email: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub logo: Option<Url>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub background: Option<Url>,
    /// Globally supported types.
    /// We consider the addon to not support any types if the list is empty.
    ///
    /// Any [`ManifestResource::Full`] in [`Manifest::resources`] which specifies id prefixes
    /// should only include prefixes that are present in the globally supported id prefixes,
    /// if they have been set, i.e. [`Option::Some`]!
    ///
    /// Types examples: `movie`, `series`, `anime`, `other`, `tv`, etc.
    pub types: Vec<String>,
    /// # Resources
    ///
    /// # Examples
    ///
    /// ## Short format
    /// ```
    /// use std::vec::Vec;
    /// use stremio_core::types::addon::ManifestResource;
    ///
    /// let manifest_resources_short = serde_json::json!({
    ///     // Addon manifest.json ....
    ///
    ///     // short-format of resources:
    ///     "resources": ["catalog", "meta", "subtitles"],
    /// }).get("resources").unwrap().clone();
    ///
    /// let resources = serde_json::from_value::<Vec<ManifestResource>>(manifest_resources_short).expect("Failed to deserialize ManifestResource");
    /// ```
    ///
    /// ## Long format
    ///
    /// ```
    /// use std::vec::Vec;
    /// use stremio_core::types::addon::ManifestResource;
    ///
    /// let manifest_resources_long = serde_json::json!({
    ///     // Addon manifest.json ....
    ///
    ///     // long-format of resources:
    ///     "resources": [
    ///         {
    ///             "name": "catalog",
    ///             // optional, i.e. can be null
    ///             "types": ["anime", "series", "movies"],
    ///             // optional, i.e. can be null
    ///             "idPrefixes": ["tt", "kitsu"],
    ///         },
    ///         "meta",
    ///         "subtitles"
    ///     ]
    /// }).get("resources").unwrap().clone();
    ///
    /// let resources = serde_json::from_value::<Vec<ManifestResource>>(manifest_resources_long).expect("Failed to deserialize ManifestResource");
    /// ```
    pub resources: Vec<ManifestResource>,
    /// Globally supported id prefixes by the addon.
    ///
    /// Any [`ManifestResource::Full`] in [`Manifest::resources`] which specifies id prefixes
    /// should only include prefixes that are present in the globally supported id prefixes,
    /// if they have been set, i.e. [`Option::Some`]!
    ///
    /// Prefixes examples: `tt`, `anidb`, `kitsu`, etc.
    pub id_prefixes: Option<Vec<String>>,
    #[serde(default)]
    #[serde_as(deserialize_as = "UniqueVec<Vec<_>, ManifestCatalogUniqueVecAdapter>")]
    pub catalogs: Vec<ManifestCatalog>,
    #[serde(default)]
    #[serde_as(deserialize_as = "UniqueVec<Vec<_>, ManifestCatalogUniqueVecAdapter>")]
    pub addon_catalogs: Vec<ManifestCatalog>,
    #[serde(default)]
    pub behavior_hints: ManifestBehaviorHints,
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
                let type_supported = types.is_some_and(|types| types.contains(&path.r#type));
                let id_supported = id_prefixes.map_or(true, |id_prefixes| {
                    id_prefixes.iter().any(|prefix| path.id.starts_with(prefix))
                });
                type_supported && id_supported
            }
        }
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
pub struct ManifestPreview {
    pub id: String,
    #[cfg_attr(test, derivative(Default(value = "Version::new(0, 0, 1)")))]
    pub version: Version,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub logo: Option<Url>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub background: Option<Url>,
    pub types: Vec<String>,
    #[serde(default)]
    pub behavior_hints: ManifestBehaviorHints,
}

impl From<&Manifest> for ManifestPreview {
    fn from(manifest_full: &Manifest) -> Self {
        Self {
            id: manifest_full.id.clone(),
            version: manifest_full.version.clone(),
            name: manifest_full.name.clone(),
            description: manifest_full.description.clone(),
            logo: manifest_full.logo.clone(),
            background: manifest_full.background.clone(),
            types: manifest_full.types.clone(),
            behavior_hints: manifest_full.behavior_hints.clone(),
        }
    }
}

/// Resources supported by the addon.
///
/// # Examples
///
/// ```
/// use stremio_core::types::addon::ManifestResource;
///
/// let short_resource = serde_json::json!("name");
/// let manifest_resource = serde_json::from_value::<ManifestResource>(short_resource).expect("Valid ManifestResource::Short");
/// assert_eq!(ManifestResource::Short("name".into()), manifest_resource);
///
/// // Full resource definition with `types`` but no `idPrefixes` defined
/// {
///     let full_resource = serde_json::json!({
///         "name": "no-idPrefixes",
///         "types": ["series", "movies"],
///     });
///     let manifest_resource = serde_json::from_value::<ManifestResource>(full_resource).expect("Valid ManifestResource::Full");
///     assert_eq!(ManifestResource::Full{ name: "no-idPrefixes".into(), types: Some(vec!["series".into(), "movies".into()]), id_prefixes: None}, manifest_resource);
///
///     // empty array for types
///     let full_resource = serde_json::json!({
///         "name": "no-idPrefixes",
///         "types": [],
///     });
///     let manifest_resource = serde_json::from_value::<ManifestResource>(full_resource).expect("Valid ManifestResource::Full");
///     assert_eq!(ManifestResource::Full{ name: "no-idPrefixes".into(), types: Some(vec![]), id_prefixes: None}, manifest_resource);
/// }
/// // Full resource definition with no `idPrefixes`` but no `types` defined
/// {
///     let full_resource = serde_json::json!({
///         "name": "no-types",
///         "idPrefixes": ["tt", "kitsu"],
///     });
///     let manifest_resource = serde_json::from_value::<ManifestResource>(full_resource).expect("Valid ManifestResource::Full");
///     assert_eq!(ManifestResource::Full { name: "no-types".into(), types: None, id_prefixes: Some(vec!["tt".into(), "kitsu".into()]) }, manifest_resource);
///
///     // empty array for `idPrefixes`
///     let full_resource = serde_json::json!({
///         "name": "no-types",
///         "idPrefixes": [],
///     });
///     let manifest_resource = serde_json::from_value::<ManifestResource>(full_resource).expect("Valid ManifestResource::Full");
///     assert_eq!(ManifestResource::Full { name: "no-types".into(), types: None, id_prefixes: Some(vec![]) }, manifest_resource);
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ManifestResource {
    /// Short resource format defines only the `name` of the resource
    Short(String),
    #[serde(rename_all = "camelCase")]
    Full {
        name: String,
        /// The types should be a subset of the ones defined in [`Manifest::types`].
        #[serde(default)]
        types: Option<Vec<String>>,
        /// The id prefixes should be a subset of the ones defined in [`Manifest::id_prefixes`].
        ///
        /// If `None`, then [`Manifest::id_prefixes`] will be used.
        #[serde(default)]
        id_prefixes: Option<Vec<String>>,
    },
}

/// Creates a [`ManifestResource::Short`] resource from a `&str`
impl From<&str> for ManifestResource {
    fn from(name: &str) -> Self {
        ManifestResource::Short(name.to_string())
    }
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCatalog {
    pub id: String,
    /// E.g. `movie`, `series`
    pub r#type: String,
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
    pub fn are_extra_names_supported(&self, extra_names: &[String]) -> bool {
        let all_supported = extra_names.iter().all(|extra_name| {
            self.extra
                .iter()
                .any(|extra_prop| &extra_prop.name == extra_name)
        });
        let required_satisfied = self
            .extra
            .iter()
            .filter(|extra_prop| extra_prop.is_required)
            .all(|extra_prop| {
                extra_names
                    .iter()
                    .any(|extra_name| extra_name == &extra_prop.name)
            });
        all_supported && required_satisfied
    }

    pub fn default_required_extra(&self) -> Option<Vec<ExtraValue>> {
        self.extra
            .iter()
            .filter(|extra| extra.is_required)
            .map(|extra| {
                extra.options.first().map(|first_option| ExtraValue {
                    name: extra.name.to_owned(),
                    value: first_option.to_owned(),
                })
            })
            .collect::<Option<Vec<_>>>()
    }
}

struct ManifestCatalogUniqueVecAdapter;

impl UniqueVecAdapter for ManifestCatalogUniqueVecAdapter {
    type Input = ManifestCatalog;
    type Output = (String, String);
    fn hash(catalog: &Self::Input) -> Self::Output {
        (catalog.id.to_owned(), catalog.r#type.to_owned())
    }
}

#[serde_as]
#[derive(Derivative, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(untagged)]
pub enum ManifestExtra {
    #[derivative(Default)]
    Full {
        #[serde(rename = "extra")]
        #[serde_as(
            deserialize_as = "UniqueVec<Vec<ExtraPropValid>, ExtraPropFullUniqueVecAdapter>"
        )]
        props: Vec<ExtraProp>,
    },
    Short {
        #[serde(default, rename = "extraRequired")]
        #[serde_as(deserialize_as = "UniqueVec<Vec<_>, ExtraPropShortUniqueVecAdapter>")]
        required: Vec<String>,
        #[serde(default, rename = "extraSupported")]
        #[serde_as(deserialize_as = "UniqueVec<Vec<_>, ExtraPropShortUniqueVecAdapter>")]
        supported: Vec<String>,
    },
}

impl ManifestExtra {
    pub fn iter(&self) -> impl Iterator<Item = Cow<ExtraProp>> {
        match &self {
            ManifestExtra::Full { props } => Either::Left(props.iter().map(Cow::Borrowed)),
            ManifestExtra::Short {
                required,
                supported,
            } => Either::Right(supported.iter().map(move |name| {
                Cow::Owned(ExtraProp {
                    name: name.to_owned(),
                    is_required: required.contains(name),
                    options: vec![],
                    options_limit: OptionsLimit::default(),
                })
            })),
        }
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtraProp {
    pub name: String,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub options: Vec<String>,

    /// The maximum options that should be passed for this addon extra property.
    #[serde(default)]
    pub options_limit: OptionsLimit,
}

struct ExtraPropFullUniqueVecAdapter;

impl UniqueVecAdapter for ExtraPropFullUniqueVecAdapter {
    type Input = ExtraProp;
    type Output = String;
    fn hash(extra_prop: &Self::Input) -> Self::Output {
        extra_prop.name.to_owned()
    }
}

struct ExtraPropShortUniqueVecAdapter;

impl UniqueVecAdapter for ExtraPropShortUniqueVecAdapter {
    type Input = String;
    type Output = String;
    fn hash(name: &Self::Input) -> Self::Output {
        name.to_owned()
    }
}

struct ExtraPropValid;

impl<'de> DeserializeAs<'de, ExtraProp> for ExtraPropValid {
    fn deserialize_as<D>(deserializer: D) -> Result<ExtraProp, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match ExtraProp::deserialize(deserializer)? {
            ExtraProp { name, .. } if name == SKIP_EXTRA_PROP.name => SKIP_EXTRA_PROP.to_owned(),
            extra_prop => extra_prop,
        })
    }
}

#[derive(Clone, Copy, Deref, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct OptionsLimit(pub usize);

impl Default for OptionsLimit {
    fn default() -> OptionsLimit {
        OptionsLimit(1)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestBehaviorHints {
    #[serde(default)]
    pub adult: bool,
    #[serde(default)]
    pub p2p: bool,
    #[serde(default)]
    pub configurable: bool,
    #[serde(default)]
    pub configuration_required: bool,
}
