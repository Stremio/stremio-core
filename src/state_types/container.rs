use super::actions::Action;
use std::cell::{Ref, RefCell};
use serde::Serialize;
use std::ops::Deref;

pub type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;
pub struct Container<S: 'static> {
    state: RefCell<S>,
    reducer: ReducerFn<S>,
}

pub trait ContainerInterface {
    fn dispatch(&self, action: &Action) -> bool;
    fn get_state_serialized(&self) -> Result<String, serde_json::Error>;
}

impl<S> Container<S> {
    pub fn with_reducer(state: S, reducer: ReducerFn<S>) -> Container<S> {
        Container {
            state: RefCell::new(state),
            reducer,
        }
    }
    pub fn get_state(&self) -> Ref<'_, S> {
        self.state.borrow()
    }
}
impl<S> ContainerInterface for Container<S>
where
    S: Serialize,
{
    fn dispatch(&self, action: &Action) -> bool {
        // WARNING: the borrow has to be dropped before borrow_mut
        let x = (self.reducer)(&self.state.borrow(), action);
        match x {
            Some(new_state) => {
                *self.state.borrow_mut() = *new_state;
                true
            }
            None => false,
        }
    }
    fn get_state_serialized(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self.get_state().deref())
    }
}
