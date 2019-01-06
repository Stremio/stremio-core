use serde_derive::*;

mod manifest;
pub use self::manifest::*;

mod meta_item;
pub use self::meta_item::*;

mod stream;
pub use self::stream::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct CatalogResponse {
    pub metas: Vec<MetaItem>,
}

// @TODO MetaResponse

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamResponse {
    pub streams: Vec<StreamItem>,
}
