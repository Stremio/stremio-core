use super::actions::Action;

pub type ReducerFn<S> = &'static Fn(&S, &Action) -> Option<Box<S>>;
pub struct Container<S: 'static> {
    state: Box<S>,
    reducer: ReducerFn<S>,
}

impl<S> Container<S> {
    pub fn with_reducer(state: S, reducer: ReducerFn<S>) -> Container<S> {
        Container{
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

