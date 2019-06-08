mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

use crate::types::addons::Descriptor;
use crate::types::api::User;
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
use futures::future;
use futures::future::Future;
// @TODO move loadable
// @TODO type aliases like in catalogs (Group, etc.)
// @TODO Arc
type Group<Item> = (ResourceRequest, Loadable<Vec<Item>, String>);
pub struct AddonAggr<Item> {
    // @TODO generic err type
    pub groups: Vec<Group<Item>>,
}
impl<Item> AddonAggr<Item> {
    pub fn new<Env: Environment + 'static>(
        addons: &[Descriptor],
        aggr_req: &AggrRequest,
    ) -> (Self, Effects) {
        let (effects, groups): (Vec<_>, Vec<_>) = aggr_req
            .plan(&addons)
            .into_iter()
            .map(|addon_req| (addon_get::<Env>(&addon_req), (addon_req, Loadable::Loading)))
            .unzip();
        (AddonAggr { groups }, Effects::many(effects))
    }
}
impl<Item> Update for AddonAggr<Item> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::AddonResponse(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| &g.0 == req) {
                    // we may need try_into on ResourceResponse
                    let group_content = Loadable::Loading; // @TODO
                    self.groups[idx] = (req.to_owned(), group_content);
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
fn addon_get<Env: Environment + 'static>(req: &ResourceRequest) -> Effect {
    // we will need that, cause we have to move it into the closure
    let req = req.clone();
    Box::new(
        Env::addon_transport(&req.transport_url)
            .get(&req.resource_ref)
            .then(move |res| match res {
                Ok(resp) => future::ok(Internal::AddonResponse(req, Ok(resp)).into()),
                Err(e) => future::err(Internal::AddonResponse(req, Err(e.to_string())).into()),
            }),
    )
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
