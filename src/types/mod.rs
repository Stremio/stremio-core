use serde_derive::*;

mod meta_item;
// @TODO: consider renaming to just 'stream'
mod stream_item;
pub use self::meta_item::*;
pub use self::stream_item::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct CatalogResponse {
    pub metas: Vec<MetaItem>,
}

// @TODO MetaResponse

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamResponse {
    pub streams: Vec<StreamItem>,
}
