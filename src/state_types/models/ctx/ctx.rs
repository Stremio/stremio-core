use super::library::LibraryLoadable;
use super::user::UserLoadable;
use crate::state_types::msg::Msg;
use crate::state_types::{Effects, Environment, Update};
use derivative::Derivative;
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Derivative, Clone, Serialize)]
#[derivative(Default, Debug)]
pub struct Ctx<Env: Environment> {
    pub user: UserLoadable,
    #[serde(skip)]
    pub library: LibraryLoadable,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        let library_effects = self.library.update::<Env>(&self.user, msg);
        let user_effects = self.user.update::<Env>(msg);
        library_effects.join(user_effects)
    }
}
