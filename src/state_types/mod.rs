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

// flow is: dispatch -> <all reactors> -> recv
// each reactor can further emit to all the following reactors
type Dispatcher = Box<Fn(&Action)>;
type Reactor = &'static Fn(&Action, &Dispatcher);
pub struct Chain {
    dispatcher: Dispatcher,
}
impl Chain {
    pub fn new(reactors: Vec<Reactor>, recv: &'static Fn(&Action)) -> Chain {
        // perhaps this might be helpful to remove the unwraps: https://www.reddit.com/r/rust/comments/64f9c8/idea_replace_with_is_it_safe/
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
