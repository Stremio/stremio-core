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
    // @TODO result
    CatalogsReceived(CatalogResponse),
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
    //Filtered(Vec<T>),
    // @TODO groups
    Grouped(Vec<T>),
}

#[derive(Debug, Serialize)]
pub struct State {
    pub catalog: Loadable<ItemsView<MetaItem>, String>,
}


// @TODO: split into another file
// @TODO decide whether we want to borrow actions or own them
type ReducerFn = &'static Fn(&State, Action) -> Option<Box<State>>;
pub struct StateContainer {
    pub state: Box<State>,
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
}
