use serde_derive::*;

mod addons;
pub use self::addons::*;

mod meta_item;
pub use self::meta_item::*;

mod stream;
pub use self::stream::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CatalogResponse {
    pub metas: Vec<MetaPreview>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MetaResponse {
    // @TODO: detailed meta item
    pub meta: MetaItem,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StreamResponse {
    pub streams: Vec<Stream>,
}
