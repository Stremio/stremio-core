use crate::state_types::*;
use std::cell::RefCell;
use std::rc::Rc;

type ContainerHolder = Rc<RefCell<ContainerInterface>>;
type Containers = Vec<(String, ContainerHolder)>;
pub struct FinalHandler {
    containers: Containers,
    cb: DispatcherFn,
}
impl FinalHandler {
    pub fn new(containers: Containers, cb: DispatcherFn) -> Self {
        FinalHandler { containers, cb }
    }
}

impl Handler for FinalHandler {
    fn handle(&self, action: &Action, _: Rc<DispatcherFn>) {
        // because our handler chain will not allow an action to be dispatched recursively to
        // ourselves, this borrow_mut() is safe
        (self.cb)(action);
        for (id, container) in self.containers.iter() {
            let has_new_state = container.borrow_mut().dispatch(action);
            if has_new_state {
                (self.cb)(&Action::NewState(id.to_owned()));
            }
        }
    }
}
