use super::actions::Action;

// Build a chain of Handlers
// the flow is: .dispatch() -> <all handlers> -> recv
// Each handler can further emit to all of it's following handlers
// This is achieved by building a recursive chain of closures, each one invoking a handler, while using the previous closure (h_taken)
// to pass the action along the chain and give the handler an ability to emit new actions from that point of the chain onward
use std::rc::Rc;
pub type DispatcherFn = Box<Fn(&Action)>;

pub trait Handler {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>);
}

pub struct Chain {
    dispatcher: DispatcherFn,
}
impl Chain {
    pub fn new(mut handlers: Vec<Box<Handler>>) -> Chain {
        let empty_cb: DispatcherFn = Box::new(|_| ());
        let dispatcher = handlers
            .drain(0..)
            .rev()
            .fold(empty_cb, |next, handler| {
                let next = Rc::new(next);
                Box::new(move |action| {
                    // propagate the action up the chain, but also allow the handler to
                    // emit actions from the same point of the chain
                    next(&action);
                    handler.handle(&action, next.clone());
                })
            });
        Chain { dispatcher }
    }
    pub fn dispatch(&self, action: &Action) {
        (self.dispatcher)(action);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;
    #[test]
    fn propagation() {
        struct Bouncer {};
        impl Handler for Bouncer {
            fn handle(&self, _: &Action, emit: Rc<DispatcherFn>) {
                emit(&Action::AddonsChanged);
            }
        }

        struct Consumer {
            pub num: RefCell(u32)
        };
        impl Handler for Consumer {
            fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
                *self.num.borrow_mut() += 1;
            }
        }
        
        let chain = Chain::new(vec![
            Box::new(Bouncer{}),
            Box::new(Bouncer{}),
            Box::new(Bouncer{}),
            Box::new(Consumer{ num: RefCell::new(0) })
        ]);
        chain.dispatch(&Action::AddonsChanged);
        assert_eq!(consumer.num, 2**3, "correct number of actions emitted");
    }
}
