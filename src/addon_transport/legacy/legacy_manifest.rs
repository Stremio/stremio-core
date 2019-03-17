use crate::types::addons::*;
use serde_derive::*;

//
// Manifest types
//
#[derive(Deserialize)]
#[serde(untagged)]
pub enum LegacyIdProperty {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyManifest {
    id: String,
    name: String,
    description: Option<String>,
    logo: Option<String>,
    background: Option<String>,
    version: semver::Version,
    methods: Vec<String>,
    types: Vec<String>,
    contact_email: Option<String>,
    id_property: Option<LegacyIdProperty>,
    // @TODO sorts
}
impl From<LegacyManifest> for Manifest {
    fn from(m: LegacyManifest) -> Self {
        let mut resources: Vec<ManifestResource> = vec![];
        let mut catalogs: Vec<ManifestCatalog> = vec![];
        let id_prefixes = m
            .id_property
            .map(|id_property| {
                let all = match id_property {
                    LegacyIdProperty::One(s) => vec![s],
                    LegacyIdProperty::Many(m) => m,
                };
                all.iter()
                    .map(|p| match &p as &str {
                        "imdb_id" => "tt".to_owned(),
                        "yt_id" => "UC".to_owned(),
                        id => format!("{}:", id),
                    })
                    .collect()
            });
        // @TODO catalogs 
        if m.methods.iter().any(|x| x == "meta.get") {
            resources.push(ManifestResource::Short("meta".into()))
        }
        if m.methods.iter().any(|x| x == "stream.find") {
            resources.push(ManifestResource::Short("stream".into()))
        }
        Manifest {
            id: m.id,
            name: m.name,
            version: m.version,
            resources,
            types: m.types,
            catalogs,
            background: m.background,
            logo: m.logo,
            id_prefixes,
            description: m.description,
            contact_email: m.contact_email,
        }
    }
}
