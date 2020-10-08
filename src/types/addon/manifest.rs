use crate::types::addon::ResourceRef;
use derivative::Derivative;
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
    pub behavior_hints: serde_json::Map<String, serde_json::Value>,
}

impl Manifest {
    pub fn is_supported(
        &self,
        ResourceRef {
            resource,
            type_name,
            id,
            extra,
        }: &ResourceRef,
    ) -> bool {
        // catalogs are a special case
        if resource == "catalog" {
            return self
                .catalogs
                .iter()
                .any(|c| &c.type_name == type_name && &c.id == id && c.is_extra_supported(&extra));
        }
        let res = match self.resources.iter().find(|res| res.name() == resource) {
            None => return false,
            Some(resource) => resource,
        };
        let types = match res {
            ManifestResource::Short(_) => Some(&self.types),
            ManifestResource::Full { types, .. } => types.as_ref(),
        };
        let id_prefixes = match res {
            ManifestResource::Short(_) => self.id_prefixes.as_ref(),
            ManifestResource::Full { id_prefixes, .. } => id_prefixes.as_ref(),
        };
        // types MUST contain type_name
        // and if there is id_prefixes, our id should start with at least one of them
        let is_types_match = types.map_or(false, |types| types.contains(type_name));
        let is_id_match = id_prefixes.map_or(true, |prefixes| {
            prefixes.iter().any(|pref| id.starts_with(pref))
        });
        is_types_match && is_id_match
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
#[serde(rename_all = "camelCase")]
pub struct ManifestCatalog {
    #[serde(rename = "type")]
    pub type_name: String,
    pub id: String,
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: ManifestExtra,
}

impl ManifestCatalog {
    pub fn extra_iter<'a>(&'a self) -> impl Iterator<Item = Cow<ManifestExtraProp>> + 'a {
        match &self.extra {
            ManifestExtra::Full { props } => Either::Left(props.iter().map(Cow::Borrowed)),
            ManifestExtra::Short {
                required,
                supported,
            } => Either::Right(supported.iter().map(move |name| {
                Cow::Owned(ManifestExtraProp {
                    name: name.to_owned(),
                    is_required: required.contains(name),
                    ..Default::default()
                })
            })),
        }
    }
    pub fn is_extra_supported(&self, extra: &[(String, String)]) -> bool {
        let all_supported = extra
            .iter()
            .all(|(k, _)| self.extra_iter().any(|e| k == &e.name));
        let requirements_satisfied = self
            .extra_iter()
            .filter(|e| e.is_required)
            .all(|e| extra.iter().any(|(k, _)| k == &e.name));
        all_supported && requirements_satisfied
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
        props: Vec<ManifestExtraProp>,
    },
    Short {
        #[serde(default, rename = "extraRequired")]
        required: Vec<String>,
        #[serde(default, rename = "extraSupported")]
        supported: Vec<String>,
    },
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
pub struct ManifestExtraProp {
    pub name: String,
    #[serde(default)]
    pub is_required: bool,
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub options_limit: OptionsLimit,
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
    fn name(&self) -> &str {
        match self {
            ManifestResource::Short(name) => name,
            ManifestResource::Full { name, .. } => name,
        }
    }
}
