use crate::state_types::msg::Msg;
use crate::state_types::Effects;

pub trait Model: Update {
    type Field;
    fn update_field(&mut self, field: &Self::Field, msg: &Msg) -> Effects;
}

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<Ctx> {
    fn update(&mut self, ctx: &Ctx, msg: &Msg) -> Effects;
}
