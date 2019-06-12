use crate::types::addons::{AggrRequest, ResourceRequest};
use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::MetaPreview;
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct CatalogGrouped {
    pub groups: Vec<ItemsGroup<Vec<MetaPreview>>>,
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
    pub item_pages: Vec<ItemsGroup<Vec<MetaPreview>>>,
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
                self.item_pages = vec![ItemsGroup::new(resource_req.to_owned())];
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
