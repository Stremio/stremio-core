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

#[derive(Debug, Serialize)]
pub enum ItemsView<T> {
    // @TODO filters
    Filtered(Vec<T>),
    // @TODO groups
    Grouped(Vec<T>),
}

// @TODO can we make it so that each property of the State is a separate reducer
#[derive(Debug, Serialize)]
pub struct State {
    pub catalog: Loadable<ItemsView<MetaItem>, String>,
}


// @TODO: split into another file
// @TODO decide whether we want to borrow actions or own them
// @TODO decide whether we should own the state or borrow it
// owning would allow us to keep some of the stuff within the old state rather than copying them
// borrowing is cleaner however, as it guarantees we cannot modify it
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
