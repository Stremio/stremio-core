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
    cb: FinalFn<T>,
}
impl<T> FinalHandler<T> {
    pub fn new(containers: Vec<(T, ContainerHolder)>, cb: FinalFn<T>) -> Self {
        FinalHandler { containers, cb }
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

// a helper to ensure that this container receives LoadWithCtx only if a previous Load has been
// dispatched with the same action
struct ContainerLoadFilter {
    container: Box<dyn ContainerInterface>,
    last_load: Option<ActionLoad>,
}
impl ContainerLoadFilter {
    pub fn new(container: Box<dyn ContainerInterface>) -> Self {
        ContainerLoadFilter {
            container,
            last_load: None,
        }
    }
}
impl ContainerInterface for ContainerLoadFilter {
    fn dispatch(&mut self, action: &Action) -> bool {
        match action {
            Action::Load(action_load) => {
                self.last_load = Some(action_load.to_owned());
            }
            Action::LoadWithCtx(_, _, action_load) => {
                if Some(action_load) != self.last_load.as_ref() {
                    return false;
                }
            }
            _ => {}
        };
        self.dispatch(action)
    }
    fn get_state_serialized(&self) -> Result<String, serde_json::Error> {
        self.container.get_state_serialized()
    }
}
