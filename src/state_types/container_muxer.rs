use crate::state_types::*;
use enclose::*;
use serde_derive::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "args")]
pub enum MuxerEvent<'a, T>
where
    T: Clone,
{
    Event(&'a Event),
    NewState(&'a T),
}

type FinalFn<T> = Box<Fn(MuxerEvent<T>)>;

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
            Box::new(enclose!((filters) move |msg| {
                if let Msg::Event(o) = msg {
                    cb(MuxerEvent::Event(o));
                }
                for (id, container) in containers.iter() {
                    // If we've set a filter for this container ID,
                    // we will only allow Load/LoadWithCtx that match it
                    if let Some(x) = filters.borrow().get(id) {
                        match msg {
                            Msg::Action(Action::Load(y)) | Msg::Internal(Internal::LoadWithCtx(_, y)) if x != y => continue,
                            _ => ()
                        }
                    }
                    let has_new_state = container.dispatch(msg);
                    if has_new_state {
                        cb(MuxerEvent::NewState(id));
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
        self.chain.dispatch(&action.to_owned().into())
    }
    pub fn dispatch_load_to(&self, id: &T, action_load: &ActionLoad) {
        self.filters
            .borrow_mut()
            .insert(id.to_owned(), action_load.to_owned());
        self.chain
            .dispatch(&Action::Load(action_load.to_owned()).into());
    }
}
