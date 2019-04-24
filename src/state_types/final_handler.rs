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

pub struct FinalHandler<T> {
    containers: Vec<(T, ContainerHolder)>,
    //filters: Vec<Option<Box<Fn(&Action) -> bool>>>,
    cb: FinalFn<T>,
}
impl<T> FinalHandler<T> {
    pub fn new(containers: Vec<(T, ContainerHolder)>, cb: FinalFn<T>) -> Self {
        FinalHandler {
            containers,
            //filters: vec![],
            cb
        }
    }
}

impl<T> Handler for FinalHandler<T> {
    fn handle(&self, action: &Action, _: Rc<DispatcherFn>) {
        // because our handler chain will not allow an action to be dispatched recursively to
        // ourselves, this borrow_mut() is safe
        (self.cb)(Event::Action(action));
        for (id, container) in self.containers.iter() {
            let has_new_state = container.borrow_mut().dispatch(action);
            if has_new_state {
                (self.cb)(Event::NewState(id));
            }
        }
    }
}

/*
// ContainerMuxer: this allows you to manage multiple containers
// @TODO drop the Rc, or at least the RefCell
struct ContainerMuxer {
    chain: Chain,
}
impl ContainerMuxer {
    pub fn new<T: 'static>(
        middlewares: Vec<Box<Handler>>,
        containers: Vec<(T, ContainerHolder)>,
        cb: FinalFn<T>
    ) -> Self {
        let mut handlers = middlewares;
        handlers.push(Box::new(FinalHandler::new(containers, cb)));
        let chain = Chain::new(handlers);
        ContainerMuxer { chain }
    }
    pub fn dispatch(&self, action: &Action) {
        self.chain.dispatch(action)
    }
    //pub fn dispatch_to<T: 'static>(&self, id: &T, action: &Action) {
    //}
}
*/
