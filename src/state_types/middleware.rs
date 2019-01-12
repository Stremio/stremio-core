use super::actions::Action;

// flow is: dispatch -> <all handlers> -> recv
// each handler can further emit to all the following handlers
type Dispatcher = Box<Fn(&Action)>;
type Handler = &'static Fn(&Action, &Dispatcher);
pub struct Chain {
    dispatcher: Dispatcher,
}
impl Chain {
    pub fn new(handlers: Vec<Handler>, recv: &'static Fn(&Action)) -> Chain {
        // perhaps this might be helpful to remove the unwraps: https://www.reddit.com/r/rust/comments/64f9c8/idea_replace_with_is_it_safe/
        let mut dispatcher: Option<Dispatcher> = Some(Box::new(move |action| recv(&action)));
        for handler in handlers.iter().rev() {
            let d_taken = dispatcher.take().unwrap();
            let h_taken = *handler;
            dispatcher = Some(Box::new(move |action| {
                h_taken(&action, &d_taken);
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
