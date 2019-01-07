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
    // @TODO this is temp, fix it
    CatalogsReceived(Result<CatalogResponse, &'static str>),
}
// Middleware actions: AddonRequest, AddonResponse

#[derive(Debug, Serialize)]
pub enum Loadable<T, M> {
    NotLoaded,
    Loading,
    Ready(T),
    Message(M),
}

/*
// @TODO: move this to FilteredCatalogs reducer
struct Filter {
    pub filter_type: String,
    pub selected: Option<String>,
}
struct FilteredContent<T> {
    filters: Vec<Filter>, // types + catalogs + catalog filters
    pages: Vec<Loadable<Vec<T>, String>>
}
*/

#[derive(Debug, Serialize)]
pub enum ItemsView<T> {
    // @TODO filters
    Filtered(Vec<T>),
    // @TODO groups
    Grouped(Vec<T>),
}

// @TODO: get rid of this; this will be defined by the reducer itself
#[derive(Debug, Serialize)]
pub struct State {
    pub catalog: Loadable<ItemsView<MetaItem>, String>,
}


// @TODO: split into another file
// @TODO borrow actions instead of owning
// @TODO generic state here
type ReducerFn = &'static Fn(&State, Action) -> Option<Box<State>>;
pub struct StateContainer {
    state: Box<State>,
    reducer: ReducerFn,
}

impl StateContainer {
    pub fn with_reducer(reducer: ReducerFn) -> StateContainer {
        StateContainer{
            state: Box::new(State{
                catalog: Loadable::NotLoaded
            }),
            reducer: reducer,
        }
    }
    pub fn dispatch(&mut self, action: Action) {
        match (self.reducer)(&self.state, action) {
            Some(new_state) => { self.state = new_state },
            None => {},
        }
    }
    pub fn get_state(&self) -> &State {
        &self.state
    }
}

// trait Middleware {
//  pub dispatch
//  @TODO better name? or the passthrough / opaque semantics should be in the construct phrase
//  pub connect_passthrough
//  pub connect_opaque
//  emit: fn
//  reactor: fn
//  }
