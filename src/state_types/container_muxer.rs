use crate::state_types::*;
use std::cell::RefCell;
use std::rc::Rc;

type ContainerHolder = Rc<RefCell<ContainerInterface>>;

#[derive(Debug)]
pub enum Event<'a, T> {
    Action(&'a Action),
    NewState(&'a T),
}

type FinalFn<T> = Box<Fn(Event<T>)>;

// ContainerMuxer: this allows you to manage multiple containers
// @TODO drop the Rc, or at least the RefCell
pub struct ContainerMuxer {
    chain: Chain,
}
impl ContainerMuxer {
    pub fn new<T: 'static>(
        middlewares: Vec<Box<Handler>>,
        containers: Vec<(T, ContainerHolder)>,
        cb: FinalFn<T>,
    ) -> Self {
        let chain = Chain::new(
            middlewares,
            Box::new(move |action| {
                // because our handler chain will not allow an action to be dispatched recursively to
                // ourselves, this borrow_mut() is safe
                cb(Event::Action(action));
                for (id, container) in containers.iter() {
                    let has_new_state = container.borrow_mut().dispatch(action);
                    if has_new_state {
                        cb(Event::NewState(id));
                    }
                }
            }),
        );
        ContainerMuxer { chain }
    }
    pub fn dispatch(&self, action: &Action) {
        self.chain.dispatch(action)
    }
    //pub fn dispatch_to<T: 'static>(&self, id: &T, action: &Action) {
    //}
}
