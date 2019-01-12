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
    reactors: Vec<Reactor>,
    output: Dispatcher,
}
impl Chain {
    pub fn new(reactors: Vec<Reactor>, output: Dispatcher) -> Rc<RefCell<Chain>> {
        Rc::new(RefCell::new(Chain {
            reactors: reactors,
            output: output,
        }))
    }
}
pub fn get_dispatcher(chain: Rc<RefCell<Chain>>, idx: usize) -> Dispatcher {
    // @TODO var naming
    if idx >= chain.borrow().reactors.len() {
        let chain_ref = chain.clone();
        return Box::new(move |action| {
            (chain_ref.borrow().output)(&action)
        })
    }
    let chain_ref = chain.clone();
    Box::new(move |action| {
        let chain = chain_ref.borrow();
        let dispatcher = get_dispatcher(chain_ref.clone(), idx+1);
        (chain.reactors[idx])(&action, &dispatcher);
        dispatcher(action);
    })
}

