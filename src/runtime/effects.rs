use crate::runtime::msg::Msg;
use derive_more::{From, IntoIterator};

use super::EnvFuture;

type Future = EnvFuture<'static, Msg>;

pub enum EffectFuture {
    Concurrent(Future),
    Sequential(Future),
}

#[derive(From)]
pub enum Effect {
    Msg(Box<Msg>),
    Future(EffectFuture),
}

/// # Examples
///
/// ```
/// use stremio_core::runtime::Effects;
///
/// let none_unchanged = Effects::none().unchanged();
/// let default = Effects::default();
///
/// assert_eq!(default.has_changed, false);
/// assert!(none_unchanged.is_empty());
/// assert!(default.is_empty());
/// assert_eq!(none_unchanged.has_changed, default.has_changed);
/// ```
#[derive(IntoIterator)]
pub struct Effects {
    #[into_iterator(owned)]
    /// The effects to be applied
    effects: Vec<Effect>,
    /// Whether or not the effects are changing something in the [`Model`].
    ///
    /// When `has_changed == true` it will mark the given model field
    /// as changed and add effect for it.
    /// This will trigger a [`RuntimeEvent::NewState`] for the [`Model`]
    /// with all it's changed fields.
    ///
    /// [`Model`]: crate::runtime::Model
    /// [`RuntimeEvent::NewState`]: crate::runtime::RuntimeEvent::NewState
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
        Effects::one(Effect::Msg(Box::new(msg)))
    }
    pub fn future(future: EffectFuture) -> Self {
        Effects::one(Effect::Future(future))
    }
    pub fn msgs(msgs: Vec<Msg>) -> Self {
        Effects::many(msgs.into_iter().map(Box::new).map(Effect::from).collect())
    }
    pub fn futures(futures: Vec<EffectFuture>) -> Self {
        Effects::many(futures.into_iter().map(Effect::from).collect())
    }

    /// mark model as not changed
    pub fn unchanged(mut self) -> Self {
        self.has_changed = false;
        self
    }
    pub fn join(mut self, mut effects: Effects) -> Self {
        self.has_changed = self.has_changed || effects.has_changed;
        self.effects.append(&mut effects.effects);
        self
    }

    /// Returns the amount of effects
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Returns whether there are any effects
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::none().unchanged()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_effects_defaults() {
        let none_unchanged = Effects::none().unchanged();
        let default = Effects::default();

        assert!(!default.has_changed);
        assert!(none_unchanged.is_empty());
        assert!(default.is_empty());
        assert_eq!(none_unchanged.has_changed, default.has_changed);
    }
}
