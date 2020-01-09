use super::library::LibraryLoadable;
use super::user_data::UserDataLoadable;
use crate::state_types::messages::Msg;
use crate::state_types::{Effects, Environment, Update};
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Ctx<Env: Environment> {
    pub user_data: UserDataLoadable,
    #[serde(skip)]
    pub library: LibraryLoadable,
    #[serde(skip)]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        let user_data_effects = self.user_data.update::<Env>(msg);
        let library_effects = self.library.update::<Env>(&self.user_data, msg);
        user_data_effects.join(library_effects)
    }
}
