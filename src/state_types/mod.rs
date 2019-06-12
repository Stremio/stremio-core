mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

mod models;
pub use self::models::*;

mod runtime;
pub use self::runtime::*;

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<Ctx> {
    fn update(&mut self, ctx: &Ctx, msg: &Msg) -> Effects;
}

use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use futures::future;
use futures::future::Future;
use msg::Internal::*;
// @TODO move loadable
// @TODO should this take &Descriptor too?
pub trait Group {
    fn new(req: ResourceRequest) -> Self;
    fn update(&mut self, res: &Result<ResourceResponse, EnvError>);
    fn addon_req(&self) -> &ResourceRequest;
}
pub fn addon_aggr_new<Env: Environment + 'static, G: Group>(
    addons: &[Descriptor],
    aggr_req: &AggrRequest,
) -> (Vec<G>, Effects) {
    let (effects, groups): (Vec<_>, Vec<_>) = aggr_req
        .plan(&addons)
        .into_iter()
        .map(|addon_req| (addon_get::<Env>(&addon_req), G::new(addon_req)))
        .unzip();
    (groups, Effects::many(effects))
}
pub fn addon_aggr_update<G: Group>(groups: &mut Vec<G>, msg: &Msg) -> Effects {
    match msg {
        Msg::Internal(AddonResponse(req, result)) => {
            if let Some(idx) = groups.iter().position(|g| g.addon_req() == req) {
                groups[idx].update(result);
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        _ => Effects::none().unchanged(),
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

// CatalogGrouped
use crate::types::MetaPreview;
use serde_derive::*;
const UNEXPECTED_RESP_MSG: &str = "unexpected ResourceResponse";
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}

use std::sync::Arc;
#[derive(Debug, Serialize, Clone)]
pub struct CatalogGroup {
    req: ResourceRequest,
    pub content: Loadable<Arc<Vec<MetaPreview>>, String>,
}
impl Group for CatalogGroup {
    fn new(req: ResourceRequest) -> Self {
        CatalogGroup {
            req,
            content: Loadable::Loading,
        }
    }
    fn update(&mut self, res: &Result<ResourceResponse, EnvError>) {
        self.content = match res {
            Ok(ResourceResponse::Metas { metas }) => Loadable::Ready(Arc::new(metas.to_owned())),
            Ok(_) => Loadable::Err(UNEXPECTED_RESP_MSG.to_string()),
            Err(e) => Loadable::Err(e.to_string()),
        };
    }
    fn addon_req(&self) -> &ResourceRequest {
        &self.req
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CatalogGrouped {
    pub groups: Vec<CatalogGroup>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogGrouped {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogGrouped { extra })) => {
                let (groups, effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllCatalogs { extra },
                );
                self.groups = groups;
                effects
            }
            _ => addon_aggr_update(&mut self.groups, msg),
        }
    }
}

//
// Filtered catalogs
// @TODO extra (filters)
// @TODO pagination
use crate::types::addons::ManifestCatalog;
#[derive(Debug, Default, Clone, Serialize)]
pub struct CatalogFiltered {
    pub item_pages: Vec<CatalogGroup>,
    pub catalogs: Vec<ManifestCatalog>,
    pub selected: Option<ResourceRequest>,
    // @TODO catalogs to be { is_selected, path, name, type }
    // is_selected will be whether the path matches selected, excluding the page
    // @TODO: extra (filters)
    // @TODO pagination; this can be done by incrementing skip in the ResourceRef when requesting
    // the next page; in LoadWithCtx, when we see that the request is for the next page, we add
    // another entry to item_pages
    // @TODO consider having `types` as well, with `is_selected`; this will just be an aggregated
    // view of `catalogs` for convenience
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered { resource_req })) => {
                // @TODO pagination
                let addons = &ctx.content.addons;
                self.catalogs = addons
                    .iter()
                    .map(|a| &a.manifest.catalogs)
                    .cloned()
                    .flatten()
                    .filter(|cat| cat.is_extra_supported(&[]))
                    .collect();
                self.item_pages = vec![CatalogGroup::new(resource_req.to_owned())];
                self.selected = Some(resource_req.to_owned());
                Effects::one(addon_get::<Env>(&resource_req))
            }
            Msg::Internal(AddonResponse(req, result))
                if Some(req) == self.selected.as_ref() =>
            {
                self.item_pages[0].update(result);
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}
