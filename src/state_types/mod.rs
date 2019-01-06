use crate::types::*;
use serde_derive::*;

#[derive(Debug, Serialize)]
// @TODO some generic way to do actions; perhaps enums should be avoided
// or alternatively we'd use a lot of From and Into in order to have separate events for the
// middlwares
pub enum Action {
    // @TODO args
    Init,
    Open,
}
// Middleware actions: AddonRequest, AddonResponse

#[derive(Debug, Serialize)]
pub enum Loadable<T> {
    NotLoaded,
    Loading,
    // @TODO Message, Content
    Ready(T)
}

// @TODO: can we do this with less generics and less nested enums

#[derive(Debug, Serialize)]
pub struct CatalogContent<T> {
    // @TODO filters
    //filters: Vec<>
    pub items: Vec<T>
}
// @TODO: more thinking on this; message should be more complicated
// should this be (Grouped, Filtered)
#[derive(Debug, Serialize)]
pub enum CatalogView<T> {
    Message(String),
    Content(CatalogContent<T>),
}

#[derive(Debug, Serialize)]
pub struct State {
    pub catalog: Loadable<CatalogView<MetaItem>>,
}
