use super::library::LibraryLoadable;
use super::streaming_server::StreamingServerLoadable;
use super::user_data::UserDataLoadable;
use crate::state_types::msg::Msg;
use crate::state_types::{Effects, Environment, Update};
use derivative::Derivative;
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Derivative, Clone, Serialize)]
#[derivative(Default, Debug)]
pub struct Ctx<Env: Environment> {
    pub user_data: UserDataLoadable,
    pub streaming_server: StreamingServerLoadable,
    #[serde(skip)]
    pub library: LibraryLoadable,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        let library_effects = self.library.update::<Env>(&self.user_data, msg);
        let user_data_effects = self.user_data.update::<Env>(msg);
        let streaming_server_effects = self.streaming_server.update::<Env>(&self.user_data, msg);
        library_effects
            .join(user_data_effects)
            .join(streaming_server_effects)
    }
}
