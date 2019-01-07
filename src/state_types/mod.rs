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

// @TODO: split into another file
// @TODO borrow actions instead of owning
type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;
pub struct StateContainer<S: 'static> {
    state: Box<S>,
    reducer: ReducerFn<S>,
}

impl<S> StateContainer<S> {
    pub fn with_reducer(state: S, reducer: ReducerFn<S>) -> StateContainer<S> {
        StateContainer{
            state: Box::new(state),
            reducer: reducer,
        }
    }
    pub fn dispatch(&mut self, action: &Action) {
        match (self.reducer)(&self.state, action) {
            Some(new_state) => { self.state = new_state },
            None => {},
        }
    }
    pub fn get_state(&self) -> &S {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogGrouped<T> {
    // @TOOD: loadable
    pub items: Vec<T>
}
