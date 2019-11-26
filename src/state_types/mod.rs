mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

mod models;
pub use self::models::*;

mod update;
pub use self::update::*;

mod runtime;
pub use self::runtime::*;

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<Ctx> {
    fn update(&mut self, ctx: &Ctx, msg: &Msg) -> Effects;
}
