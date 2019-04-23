use crate::state_types::*;
use std::cell::RefCell;
use std::rc::Rc;

type ContainerHolder = Rc<RefCell<ContainerInterface>>;

pub struct FinalHandler<T> {
    containers: Vec<(T, ContainerHolder)>,
    cb: DispatcherFn,
}
impl<T> FinalHandler<T> {
    pub fn new(containers: Vec<(T, ContainerHolder)>, cb: DispatcherFn) -> Self {
        FinalHandler { containers, cb }
    }
}

impl<T> Handler for FinalHandler<T> {
    fn handle(&self, action: &Action, _: Rc<DispatcherFn>) {
        // because our handler chain will not allow an action to be dispatched recursively to
        // ourselves, this borrow_mut() is safe
        (self.cb)(action);
        for (_, container) in self.containers.iter() {
            let has_new_state = container.borrow_mut().dispatch(action);
            if has_new_state {
                // @TODO
                //(self.cb)(&Action::NewState(id.to_owned()));
            }
        }
    }
}

/*
// ContainerMuxer: this allows you to manage multiple containers
struct ContainerMuxer {
    containers: Vec<(T, ContainerHolder)>,
    chain: Chain,
}
impl ContainerMuxer {
    pub fn new<T: 'static>(
        middlewares: Vec<Box<Handler>>,
        containers: Vec<(T, ContainerHolder)>,
        cb: DispatcherFn
    ) -> Self {
        let mut handlers = middlewares;
        handlers.push(Box::new(FinalHandler::new(containers, cb)));
        let chain = Chain::new(handlers);
        ContainerMuxer { chain }
    }
    pub fn dispatch(&self, action: &Action) {
        self.chain.dispatch(action)
    }
    pub fn dispatch_to(&self, id: &T, action: &Action) {

    }
}
*/
