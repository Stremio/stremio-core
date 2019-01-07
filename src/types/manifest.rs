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
pub struct ManifestCatalog {
    #[serde(rename = "type")]
    pub catalog_type: String,
    pub id: String,
    pub name: Option<String>,
    // @TODO: extraSupported, extraRequired, filters
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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
