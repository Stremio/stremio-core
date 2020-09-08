use crate::runtime::msg::Msg;
use futures::future;
use futures::future::{FutureExt, LocalBoxFuture};

pub type Effect = LocalBoxFuture<'static, Msg>;

pub struct Effects {
    effects: Vec<Effect>,
    pub has_changed: bool,
}

impl IntoIterator for Effects {
    type Item = Effect;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.effects.into_iter()
    }
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
        Effects::one(future::ready(msg).boxed_local())
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
