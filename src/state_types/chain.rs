use super::msg::Msg;

// Build a chain of Handlers
// the flow is: .dispatch() -> <all handlers> -> recv
// Each handler can further emit to all of it's following handlers
// This is achieved by building a recursive chain of closures, each one invoking a handler, while using the previous closure (h_taken)
// to pass the action along the chain and give the handler an ability to emit new actions from that point of the chain onward
use std::rc::Rc;
pub type DispatcherFn = Box<dyn Fn(&Msg)>;

pub trait Handler {
    fn handle(&self, msg: &Msg, emit: Rc<DispatcherFn>);
}

pub struct Chain {
    dispatcher: DispatcherFn,
}
impl Chain {
    pub fn new(handlers: Vec<Box<dyn Handler>>, cb: DispatcherFn) -> Chain {
        let dispatcher = handlers.into_iter().rev().fold(cb, |next, handler| {
            let next = Rc::new(next);
            Box::new(move |msg| {
                // propagate the action up the chain, but also allow the handler to
                // emit actions from the same point of the chain
                next(&msg);
                handler.handle(&msg, next.clone());
            })
        });
        Chain { dispatcher }
    }
    pub fn dispatch(&self, action: &Msg) {
        (self.dispatcher)(action);
    }
}
