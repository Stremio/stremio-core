use serde_derive::*;

mod state_container;
pub use self::state_container::*;

mod catalogs;
pub use self::catalogs::*;

mod actions;
pub use self::actions::*;

#[derive(Debug, Serialize)]
pub enum Loadable<T, M> {
    NotLoaded,
    Loading,
    Ready(T),
    Message(M),
}

use std::cell::RefCell;
use std::rc::Rc;
// flow is: dispatch -> <reactor> -> emitter
// where the emitter feeds back into dispatch
type Dispatcher = Box<Fn(&Action)>;
type Reactor = &'static Fn(&Action, &Dispatcher);
pub struct Chain {
    dispatcher: Dispatcher,
}
impl Chain {
    // @TODO: think of getting rid of the unwraps; perhaps use a Cell?
    pub fn new(reactors: Vec<Reactor>, recv: &'static Fn(&Action)) -> Chain {
        let mut dispatcher: Option<Dispatcher> = Some(Box::new(move |action| recv(&action)));
        for reactor in reactors.iter().rev() {
            let d_taken = dispatcher.take().unwrap();
            let r_taken = *reactor;
            dispatcher = Some(Box::new(move |action| {
                r_taken(&action, &d_taken);
                d_taken(&action);
            }));
        }
        Chain {
            dispatcher: dispatcher.unwrap(),
        }
    }
    pub fn dispatch(&self, action: &Action) {
        (self.dispatcher)(action);
    }
}
