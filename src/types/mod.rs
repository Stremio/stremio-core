use serde_derive::*;

mod meta_item;
// @TODO: consider renaming to just 'stream'
mod stream_item;
pub use self::meta_item::*;
pub use self::stream_item::*;

#[derive(Deserialize, Debug)]
pub struct MetaResponse {
    pub metas: Vec<MetaItem>,
}

#[derive(Deserialize, Debug)]
pub struct StreamResponse {
    pub streams: Vec<StreamItem>,
}
