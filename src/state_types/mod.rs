use crate::types::*;

#[derive(Debug)]
pub enum Action { Init }

#[derive(Debug)]
pub enum Loadable<T> {
    NotLoaded,
    Loading,
    Ready(T)
}

// @TODO: can we do this with less generics and less nested enums

#[derive(Debug)]
pub struct CatalogContent<T> {
    // @TODO filters
    //filters: Vec<>
    pub items: Vec<T>
}
// @TODO: more thinking on this; message should be more complicated
#[derive(Debug)]
pub enum CatalogView<T> {
    Message(String),
    Content(CatalogContent<T>),
}

#[derive(Debug)]
pub struct State {
    pub catalog: Loadable<CatalogView<MetaItem>>,
}
