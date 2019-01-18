use serde_derive::*;

// https://serde.rs/string-or-struct.html
use std::fmt;
use std::str::FromStr;
use std::marker::PhantomData;
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestResource {
    pub name: String,
    pub types: Option<Vec<String>>,
    pub id_prefixes: Option<Vec<String>>,
}
impl FromStr for ManifestResource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ManifestResource {
            name: s.to_string(),
            types: None,
            id_prefixes: None,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestCatalog {
    #[serde(rename = "type")]
    pub type_name: String,
    pub id: String,
    pub name: Option<String>,
    // @TODO: extraSupported, extraRequired, filters
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub background: Option<String>,
    // @TODO catalogs
    #[serde(deserialize_with = "vec_manifest_resource")]
    pub resources: Vec<ManifestResource>,
    pub types: Option<Vec<String>>,
    pub id_prefixes: Option<Vec<String>>,
    // @TODO: more efficient data structure?
    //pub behavior_hints: Vec<String>,
    #[serde(default)]
    pub catalogs: Vec<ManifestCatalog>,
}

impl AddonManifest {
    // @TODO: test
    pub fn is_supported(&self, resource_name: String, type_name: String, id: String) -> bool {
        // catalogs are a special case
        if resource_name == "catalog" {
            return self.catalogs.iter()
                .any(|c| c.type_name == type_name && c.id == id)
        }
        let resource = match self.resources.iter().find(|&res| res.name == resource_name) {
            None => return false,
            Some(resource) => resource
        };
        // types MUST contain type_name
        // and if there is id_prefixes, our id should start with at least one of them
        let is_types_match = resource.types
            .as_ref().or_else(|| self.types.as_ref())
            .map_or(false, |types| {
                types.contains(&type_name)
            });
        let is_id_match = resource.id_prefixes
            .as_ref().or_else(|| self.id_prefixes.as_ref())
            .map_or(true, |prefixes| {
                prefixes.iter().any(|pref| id.starts_with(pref))
            });
        is_types_match && is_id_match
    }
}

// @TODO: this also needs to be a crate, kind of: https://github.com/serde-rs/serde/issues/723
fn vec_manifest_resource<'de, D>(deserializer: D) -> Result<Vec<ManifestResource>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "string_or_struct")] ManifestResource);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a).collect())
}
// @TODO: move string_or_struct to a crate
fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = ()>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = ()>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, visitor: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
