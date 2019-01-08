use super::actions::Action;

pub type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;
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

