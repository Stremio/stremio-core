use super::actions::Action;
use std::cell::{Ref, RefCell};
use serde::Serialize;

pub type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;

pub trait ContainerInterface {
    fn dispatch(&self, action: &Action) -> bool;
    fn get_state_serialized(&self) -> Result<String, serde_json::Error>;
}


pub struct Container<S: 'static> {
    state: S,
    reducer: ReducerFn<S>,
}

impl<S> Container<S> {
    pub fn with_reducer(state: S, reducer: ReducerFn<S>) -> Container<S> {
        Container { state, reducer }
    }
    pub fn get_state(&self) -> &S {
        &self.state
    }
    pub fn dispatch(&mut self, action: &Action) -> bool {
        match (self.reducer)(&self.state, action) {
            Some(new_state) => {
                self.state = *new_state;
                true
            }
            None => false
        }
    }
}

pub struct ContainerHolder<S: 'static>(RefCell<Container<S>>);

impl<S> ContainerHolder<S> {
    pub fn new(container: Container<S>) -> Self {
        ContainerHolder(RefCell::new(container))
    }
    pub fn borrow_state(&self) -> Ref<'_, S> {
        Ref::map(self.0.borrow(), |m| &m.state)
    }
}

impl<S> ContainerInterface for ContainerHolder<S>
where
    S: Serialize,
{
    fn dispatch(&self, action: &Action) -> bool {
        self.0.borrow_mut().dispatch(action)
    }
    fn get_state_serialized(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.0.borrow().get_state())
    }
}
