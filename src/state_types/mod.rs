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

use crate::types::addons::{AggrRequest, ResourceRequest};
// @TODO move loadable
// @TODO type aliases like in catalogs
pub struct AddonAggr<Item> {
    // @TODO generic err type
    pub groups: Vec<(ResourceRequest, Loadable<Vec<Item>, String>)>
}
impl<Item> AddonAggr<Item> {
    pub fn new(addons: &[Descriptor], aggr_req: &AggrRequest) -> (Self, Effects) {
        let groups = aggr_req
            .plan(&addons)
            .into_iter()
            .map(|addon_req| (addon_req, Loadable::Loading))
            .collect();
        (
            AddonAggr { groups },
            // @TODO effects; we can probably do that with unzip
            Effects::none()
        )
    }
}
impl<Item> Update for AddonAggr<Item> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::AddonResponse(req, result)) => {
                // @TODO
                Effects::none()
            },
            _ => Effects::none().unchanged()
        }
    }
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
