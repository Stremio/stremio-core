use crate::runtime::msg::Msg;
use crate::runtime::Effects;

pub trait Model: Update {
    type Field;
    fn update_field(&mut self, msg: &Msg, field: &Self::Field) -> Effects;
}

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<Ctx> {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects;
}
