mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

mod models;
pub use self::models::*;

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx {
    type Ctx;
    fn update(&mut self, ctx: &Self::Ctx, msg: &Msg) -> Effects;
}

use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use futures::future;
use futures::future::Future;
use msg::Internal::*;
// @TODO move loadable
// @TODO should this take &Descriptor too?
pub trait Group {
    fn new(req: ResourceRequest) -> Self;
    // @TODO generic err type
    fn update(&mut self, resp: &Result<ResourceResponse, EnvError>) -> Self;
    fn addon_req(&self) -> &ResourceRequest;
}
pub struct AddonAggr<G: Group> {
    pub groups: Vec<G>,
}
impl<G: Group> AddonAggr<G> {
    pub fn new<Env: Environment + 'static>(
        addons: &[Descriptor],
        aggr_req: &AggrRequest,
    ) -> (Self, Effects) {
        let (effects, groups): (Vec<_>, Vec<_>) = aggr_req
            .plan(&addons)
            .into_iter()
            .map(|addon_req| (addon_get::<Env>(&addon_req), G::new(addon_req)))
            .unzip();
        (AddonAggr { groups }, Effects::many(effects))
    }
}
impl<G: Group> Update for AddonAggr<G> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(AddonResponse(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| g.addon_req() == req) {
                    self.groups[idx].update(result);
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
        Env::addon_transport(&req.base)
            .get(&req.path)
            .then(move |res| match res {
                Ok(_) => future::ok(AddonResponse(req, Box::new(res)).into()),
                Err(_) => future::err(AddonResponse(req, Box::new(res)).into()),
            }),
    )
}
