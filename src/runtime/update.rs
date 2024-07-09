use crate::models::ctx::Ctx;
use crate::runtime::msg::Msg;
use crate::runtime::{Effect, Effects, Env};
#[cfg(debug_assertions)]
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

pub trait Model<E: Env>: Clone {
    #[cfg(not(debug_assertions))]
    type Field: Send + Sync + Serialize + for<'de> Deserialize<'de>;
    #[cfg(debug_assertions)]
    type Field: Debug + Send + Sync + Serialize + for<'de> Deserialize<'de>;

    fn update(&mut self, msg: &Msg) -> (Vec<Effect>, Vec<Self::Field>);
    fn update_field(&mut self, msg: &Msg, field: &Self::Field) -> (Vec<Effect>, Vec<Self::Field>);
}

pub trait Update<E: Env> {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<E: Env> {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects;
}
