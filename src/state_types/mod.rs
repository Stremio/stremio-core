mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

use crate::types::api::User;
use crate::types::addons::Descriptor;
#[derive(Debug)]
pub struct Context {
    pub user: Option<User>,
    pub addons: Vec<Descriptor>,
    // @TODO settings
}

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx {
    fn update(&mut self, ctx: &Context, msg: &Msg) -> Effects;
}


// @TODO everything underneath will be dropped with the Elm architecture rewrite
mod container;
pub use self::container::*;

mod catalogs;
pub use self::catalogs::*;

mod chain;
pub use self::chain::*;

mod container_muxer;
pub use self::container_muxer::*;
