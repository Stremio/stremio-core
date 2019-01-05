use serde_derive::*;

mod meta_item;
pub use self::meta_item::*;

#[derive(Deserialize, Debug)]
pub struct MetaResponse {
    pub metas: Vec<MetaItem>,
}
