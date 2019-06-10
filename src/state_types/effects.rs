use super::msg::Msg;
use futures::{future, Future};

pub type Effect = Box<dyn Future<Item = Msg, Error = Msg>>;

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

    pub fn one(x: Effect) -> Self {
        Effects {
            effects: vec![x],
            has_changed: true,
        }
    }

    pub fn many(effects: Vec<Effect>) -> Self {
        Effects {
            effects,
            has_changed: true,
        }
    }

    pub fn msg(x: Msg) -> Self {
        Effects::one(Box::new(future::ok(x)))
    }

    pub fn unchanged(mut self) -> Self {
        self.has_changed = false;
        self
    }

    pub fn join(mut self, mut x: Effects) -> Self {
        self.has_changed = self.has_changed || x.has_changed;
        self.effects.append(&mut x.effects);
        self
    }
}
