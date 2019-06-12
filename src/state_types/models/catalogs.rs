use crate::types::addons::{AggrRequest, ResourceRequest, ResourceResponse};
use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;

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
