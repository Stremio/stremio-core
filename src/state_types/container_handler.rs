use crate::state_types::*;
use std::cell::RefCell;
use std::rc::Rc;

type ContainerHolder<T> = Rc<RefCell<Container<T>>>;

pub struct ContainerHandler<T: 'static> {
    id: usize,
    container: ContainerHolder<T>,
}
impl<T> ContainerHandler<T> {
    pub fn new(id: usize, container: ContainerHolder<T>) -> Self {
        ContainerHandler { id, container }
    }
}
impl<T> Handler for ContainerHandler<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // because our handler chain will not allow an action to be dispatched recursively to
        // ourselves, this borrow_mut() is safe
        let has_new_state = self.container.borrow_mut().dispatch(action).is_some();
        if has_new_state {
            emit(&Action::NewState(self.id));
        }
    }
}
