use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRequest, ResourceRef, ResourceResponse};
use crate::types::MetaPreview;
use itertools::*;
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
//
#[derive(Serialize, Clone, Debug)]
pub struct TypeEntry {
    pub type_name: String,
    pub is_selected: bool,
    //pub load: ResourceRequest,
}

#[derive(Serialize, Clone, Debug)]
pub struct CatalogEntry {
    pub name: String,
    pub is_selected: bool,
    pub load: ResourceRequest,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CatalogFiltered {
    // @TODO more sophisticated error, such as EmptyContent/UninstalledAddon/Offline
    // see https://github.com/Stremio/stremio/issues/402
    pub content: Loadable<Vec<MetaPreview>, String>,
    pub types: Vec<TypeEntry>,
    pub catalogs: Vec<CatalogEntry>,
    pub selected: Option<ResourceRequest>,
    // @TODO types to be { load_msg, is_selected, type_name }
    // @TODO catalogs to be { load_msg, is_selected, name, type }
    // is_selected will be whether the path matches selected, excluding the `skip` (page)
    // @TODO: extra (filters); there should be .extra, of all selectable extra props; consider that
    // some can be defaulted
    // @TODO pagination; this can be done by incrementing skip in the ResourceRequest when requesting
    // the next page; we will have .load_next/.load_prev (Option<ResourceRequest>) to go to next/prev
    // page
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered { resource_req })) => {
                // @TODO catalog by types
                // @TODO pagination
                let addons = &ctx.content.addons;
                self.catalogs = addons
                    .iter()
                    // this will weed out catalogs that require extra props
                    // @TODO this will be a filter_map when we generate the
                    // { load, is_selected, .. } format
                    .flat_map(|a| a.manifest.catalogs.iter().filter_map(move |cat| {
                        // Is not required, or has provided possible options (we can default to
                        // the first one)
                        let is_supported = cat.extra_iter().all(|e| {
                            !e.is_required || e.options.as_ref().map_or(false, |o| !o.is_empty())
                        });
                        let load = ResourceRequest {
                            base: a.transport_url.to_owned(),
                            path: ResourceRef::without_extra("catalog", &cat.type_name, &cat.id)
                        };
                        Some(CatalogEntry {
                            name: cat.name.as_ref().unwrap_or(&a.manifest.name).to_owned(),
                            is_selected: load.eq_no_extra(resource_req),
                            load
                        })
                    }))
                    .collect();
                // The alternative to the HashSet is to sort and dedup
                // but we want to preserve the original order in which types appear in
                self.types = self
                    .catalogs
                    .iter()
                    .map(|x| x.load.path.type_name.clone())
                    .unique()
                    .map(|type_name| TypeEntry {
                        is_selected: resource_req.path.type_name == type_name,
                        type_name,
                    })
                    .collect();
                self.content = Loadable::Loading;
                self.selected = Some(resource_req.to_owned());
                Effects::one(addon_get::<Env>(&resource_req))
            }
            Msg::Internal(AddonResponse(req, result))
                if Some(req) == self.selected.as_ref() && self.content == Loadable::Loading =>
            {
                self.content = match result.as_ref() {
                    Ok(ResourceResponse::Metas { metas }) => Loadable::Ready(metas.to_owned()),
                    Ok(_) => Loadable::Err(UNEXPECTED_RESP_MSG.to_owned()),
                    Err(e) => Loadable::Err(e.to_string()),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}
