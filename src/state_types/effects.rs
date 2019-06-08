use super::msg::Msg;
use futures::Future;

type Effect = Box<Future<Item = Msg, Error = Msg>>;

struct Effects {
    pub effects: Vec<Effect>,
    pub has_changed: bool,
}

impl Effects {
    fn none() -> Self {
        Effects {
            effects: vec![],
            has_changed: true,
        }
    }

    fn one(x: Effect) -> Self {
        Effects {
            effects: vec![x],
            has_changed: true,
        }
    }

    fn many(effects: Vec<Effect>) -> Self {
        Effects {
            effects,
            has_changed: true,
        }
    }

    fn unchanged(&mut self) -> &mut Self {
        self.has_changed = false;
        self
    }

    fn join(&mut self, mut x: Effects) -> &mut Self {
        self.has_changed = self.has_changed || x.has_changed;
        self.effects.append(&mut x.effects);
        self
    }
}
