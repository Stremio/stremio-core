use crate::types::addon::{Manifest, ManifestCatalog, ManifestResource};
use semver::Version;
use serde::Deserialize;
use url::Url;

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
pub struct LegacySort {
    name: Option<String>,
    #[serde(rename = "prop")]
    id: String,
    types: Option<Vec<String>>,
}

// A wrapper is needed cause of the way the legacy
// system returns the results
// the JSON RPC response contains { manifest, methods }
// We don't need methods cause we have them in the manifest
#[derive(Deserialize)]
pub struct LegacyManifestResp {
    manifest: LegacyManifest,
}
impl From<LegacyManifestResp> for Manifest {
    fn from(resp: LegacyManifestResp) -> Self {
        resp.manifest.into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyManifest {
    id: String,
    name: String,
    description: Option<String>,
    logo: Option<String>,
    background: Option<String>,
    version: Version,
    methods: Vec<String>,
    types: Vec<String>,
    contact_email: Option<String>,
    id_property: Option<LegacyIdProperty>,
    sorts: Option<Vec<LegacySort>>,
}
impl From<LegacyManifest> for Manifest {
    fn from(m: LegacyManifest) -> Self {
        // Catalogs: if there are sorts, add a catalog for each type for each sort
        // if there are no sorts, do that just for the types
        let types = m.types.to_owned();
        let is_find = m.methods.iter().any(|x| x == "meta.find");
        let catalogs: Vec<ManifestCatalog> = if is_find {
            match m.sorts {
                Some(sorts) => sorts
                    .iter()
                    .flat_map(|sort| {
                        let types = sort.types.as_ref().unwrap_or(&types);
                        types.iter().cloned().map(move |t| ManifestCatalog {
                            r#type: t,
                            id: sort.id.to_owned(),
                            name: sort.name.to_owned(),
                            extra: Default::default(),
                        })
                    })
                    .collect(),
                None => types
                    .iter()
                    .map(|t| ManifestCatalog {
                        r#type: t.to_owned(),
                        id: "top".to_owned(),
                        name: None,
                        extra: Default::default(),
                    })
                    .collect(),
            }
        } else {
            vec![]
        };

        // id_prefixes: the previous id_property is pretty much equivalent,
        // with the following differences:
        let id_prefixes = m.id_property.map(|id_property| {
            let all = match id_property {
                LegacyIdProperty::One(s) => vec![s],
                LegacyIdProperty::Many(m) => m,
            };
            all.iter()
                .map(|p| match p as &str {
                    "imdb_id" => "tt".to_owned(),
                    "yt_id" => "UC".to_owned(),
                    id => format!("{id}:"),
                })
                .collect()
        });

        // resources: only those two are supported by the legacy mapper
        let mut resources: Vec<ManifestResource> = vec![];
        if m.methods.iter().any(|x| x == "meta.get") {
            resources.push(ManifestResource::Short("meta".into()))
        }
        if m.methods.iter().any(|x| x == "stream.find") {
            resources.push(ManifestResource::Short("stream".into()))
        }
        if m.methods.iter().any(|x| x == "subtitles.get") {
            resources.push(ManifestResource::Short("subtitles".into()))
        }

        Manifest {
            id: m.id,
            name: m.name,
            version: m.version,
            resources,
            types: m.types,
            catalogs,
            addon_catalogs: vec![],
            background: m
                .background
                .and_then(|background| Url::parse(&background).ok()),
            logo: m.logo.and_then(|logo| Url::parse(&logo).ok()),
            id_prefixes,
            description: m.description,
            contact_email: m.contact_email,
            // The JS implementation infers `adult`, but it's such a rare case, it's not worth it:
            // https://github.com/Stremio/stremio-addon-client/blob/4f4dbbf55498d7fdc6bd41bf49cb2f05915b3f8e/lib/transports/legacy/mapper.js#L70
            behavior_hints: Default::default(),
        }
    }
}
