use crate::models::ctx::Ctx;
use crate::runtime::msg::Msg;
use crate::runtime::{Effects, Env};

pub trait Model<E: Env>: Update<E> {
    type Field;
    fn update_field(&mut self, msg: &Msg, field: &Self::Field) -> Effects;
}

pub trait Update<E: Env> {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<E: Env> {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects;
}
