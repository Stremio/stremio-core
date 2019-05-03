use crate::types::addons::ResourceRef;
use either::Either;
use semver::Version;
use serde_derive::*;
use std::borrow::Cow;

// Resource descriptors
// those define how a resource may be requested
// the tricky thing here is that resources may either be provided in a short notation, e.g. `"stream"`
// or long e.g. `{ name: "stream", types: ["movie"], id_prefixes: ["tt"] }`
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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
            ManifestResource::Short(n) => n,
            ManifestResource::Full { name, .. } => name,
        }
    }
}

// Extra descriptors
// those define the extra properties that may be passed for a catalog
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OptionsLimit(u32);
impl Default for OptionsLimit {
    fn default() -> OptionsLimit {
        OptionsLimit(1)
    }
}
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManifestExtraProp {
    pub name: String,
    #[serde(default)]
    pub is_required: bool,
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub options_limit: OptionsLimit,
}
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ManifestExtra {
    Full {
        #[serde(rename = "extra")]
        props: Vec<ManifestExtraProp>,
    },
    // Short notation, which was the standard before v3.1 protocol: https://github.com/Stremio/stremio-addon-sdk/milestone/1
    // kind of obsolete, but addons may use it
    Short {
        #[serde(default, rename = "extraRequired")]
        required: Vec<String>,
        #[serde(default, rename = "extraSupported")]
        supported: Vec<String>,
    },
}
impl Default for ManifestExtra {
    fn default() -> Self {
        ManifestExtra::Full { props: vec![] }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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
            ManifestExtra::Full { ref props } => {
                Either::Left(props.iter().map(|x| Cow::Borrowed(x)))
            }
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

// The manifest itself
// @TODO consider separating the meta in .meta (#[serde(flatten)]), in order to be able to
// construct a manifest from meta + addon builder separately
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    pub version: Version,
    pub name: String,
    pub contact_email: Option<String>,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub background: Option<String>,
    pub resources: Vec<ManifestResource>,
    pub types: Vec<String>,
    pub id_prefixes: Option<Vec<String>>,
    #[serde(default)]
    pub catalogs: Vec<ManifestCatalog>,
    // @TODO: more efficient data structure?
    //pub behavior_hints: Vec<String>,
}

impl Manifest {
    // @TODO: test
    // assert_eq!(cinemeta_m.is_supported("meta", "movie", "tt0234"), true);
    // assert_eq!(cinemeta_m.is_supported("meta", "movie", "somethingElse"), false));
    // assert_eq!(cinemeta_m.is_supported("stream", "movie", "tt0234"), false);
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
