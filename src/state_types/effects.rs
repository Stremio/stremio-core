use crate::state_types::msg::Msg;
use core::pin::Pin;
use futures::{future, Future};

pub type Effect = Pin<Box<dyn Future<Output = Msg> + Unpin>>;

pub struct Effects {
    pub effects: Vec<Effect>,
    pub has_changed: bool,
}

impl Effects {
    pub fn none() -> Self {
        Effects {
            effects: vec![],
            has_changed: true,
        }
    }

    pub fn one(effect: Effect) -> Self {
        Effects {
            effects: vec![effect],
            has_changed: true,
        }
    }

    pub fn many(effects: Vec<Effect>) -> Self {
        Effects {
            effects,
            has_changed: true,
        }
    }

    pub fn msg(msg: Msg) -> Self {
        Effects::one(Pin::new(Box::new(future::ready(msg))))
    }

    pub fn unchanged(mut self) -> Self {
        self.has_changed = false;
        self
    }

    pub fn join(mut self, mut effects: Effects) -> Self {
        self.has_changed = self.has_changed || effects.has_changed;
        self.effects.append(&mut effects.effects);
        self
    }
}
