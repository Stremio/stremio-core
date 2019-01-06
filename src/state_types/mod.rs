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
pub enum Loadable<T, M> {
    NotLoaded,
    Loading,
    Ready(T),
    Message(M),
}

#[derive(Debug, Serialize)]
pub enum ItemsView<T> {
    // @TODO filters
    Filtered(Vec<T>),
    // @TODO groups
    Grouped(Vec<T>),
}

#[derive(Debug, Serialize)]
pub struct State {
    pub catalog: Loadable<ItemsView<MetaItem>, String>,
}


// @TODO: split into another file
