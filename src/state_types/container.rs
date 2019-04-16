use super::actions::Action;
use serde::Serialize;

pub type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;
pub struct Container<S: 'static> {
    state: Box<S>,
    reducer: ReducerFn<S>,
}

pub trait ContainerInterface {
    fn dispatch(&mut self, action: &Action) -> bool;
    fn get_state_serialized(&self) -> Result<String, serde_json::Error>;
}

impl<S> Container<S> {
    pub fn with_reducer(state: S, reducer: ReducerFn<S>) -> Container<S> {
        Container {
            state: Box::new(state),
            reducer,
        }
    }
    pub fn get_state(&self) -> &S {
        &self.state
    }
}
impl<S> ContainerInterface for Container<S>
where
    S: Serialize,
{
    fn dispatch(&mut self, action: &Action) -> bool {
        match (self.reducer)(&self.state, action) {
            Some(new_state) => {
                self.state = new_state;
                true
            }
            None => false,
        }
    }
    fn get_state_serialized(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.state)
    }
}
