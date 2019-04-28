use crate::state_types::*;
use enclose::*;
use serde_derive::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum Event<'a, T>
where
    T: Clone,
{
    Action(&'a Action),
    NewState(&'a T),
}

type FinalFn<T> = Box<Fn(Event<T>)>;

// ContainerMuxer: this allows you to manage multiple containers
pub struct ContainerMuxer<U> {
    chain: Chain,
    filters: Rc<RefCell<BTreeMap<U, ActionLoad>>>,
}
impl<T> ContainerMuxer<T>
where
    T: 'static + Clone + Ord,
{
    pub fn new<U: 'static + Deref<Target = ContainerInterface>>(
        middlewares: Vec<Box<Handler>>,
        containers: Vec<(T, U)>,
        cb: FinalFn<T>,
    ) -> Self {
        let filters = Rc::new(RefCell::new(BTreeMap::new()));
        let chain = Chain::new(
            middlewares,
            Box::new(enclose!((filters) move |action| {
                cb(Event::Action(action));
                for (id, container) in containers.iter() {
                    // If we've set a filter for this container ID,
                    // we will only allow Load/LoadWithCtx that match it
                    match action {
                        Action::Load(x) | Action::LoadWithCtx(_, x) => {
                            match filters.borrow().get(id) {
                                Some(y) if x != y => continue,
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                    let has_new_state = container.dispatch(action);
                    if has_new_state {
                        cb(Event::NewState(id));
                    }
                }
            })),
        );
        ContainerMuxer { chain, filters }
    }
    pub fn dispatch(&self, action: &Action) {
        if let Action::Load(_) = action {
            self.filters.borrow_mut().clear();
        }
        self.chain.dispatch(action)
    }
    pub fn dispatch_load_to(&self, id: &T, action_load: &ActionLoad) {
        self.filters
            .borrow_mut()
            .insert(id.to_owned(), action_load.to_owned());
        self.chain.dispatch(&Action::Load(action_load.to_owned()));
    }
}
