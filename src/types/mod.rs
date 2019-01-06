use serde_derive::*;

mod manifest;
pub use self::manifest::*;
//@ TODO: should this be in a separate file
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonDescriptor {
    pub manifest: AddonManifest,
    pub transport_url: String,
    pub transport_name: String,
    // @TODO flags
}

mod meta_item;
pub use self::meta_item::*;

mod stream;
pub use self::stream::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct CatalogResponse {
    pub metas: Vec<MetaItem>,
}

pub struct MetaResponse {
    // @TODO: detailed meta item
    pub meta: MetaItem,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamResponse {
    pub streams: Vec<Stream>,
}
