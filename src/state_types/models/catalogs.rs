use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRequest, ResourceRef, ExtraProp, ResourceResponse};
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
    pub load: ResourceRequest,
}

#[derive(Serialize, Clone, Debug)]
pub struct CatalogEntry {
    pub name: String,
    pub is_selected: bool,
    pub load: ResourceRequest,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CatalogFiltered {
    pub types: Vec<TypeEntry>,
    pub catalogs: Vec<CatalogEntry>,
    pub selected: Option<ResourceRequest>,
    // @TODO more sophisticated error, such as EmptyContent/UninstalledAddon/Offline
    // see https://github.com/Stremio/stremio/issues/402
    pub content: Loadable<Vec<MetaPreview>, String>,
    // @TODO: extra (filters); there should be .extra, of all selectable extra props
    // @TODO pagination; this can be done by incrementing skip in the ResourceRequest when requesting
    // the next page; we will have .load_next/.load_prev (Option<ResourceRequest>) to go to next/prev
    // page
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered { resource_req })) => {
                // @TODO pagination
                let addons = &ctx.content.addons;
                // Catalogs are NOT filtered by type, cause the UI gets to decide whether to
                // only show catalogs for the selected type, or all of them
                self.catalogs = addons
                    .iter()
                    .flat_map(|a| a.manifest.catalogs.iter().filter_map(move |cat| {
                        // Required properties are allowed, but only if there's .options
                        // with at least one option inside (that we default to)
                        // If there are no required properties at all, this will resolve to Some([])
                        let props = cat
                            .extra_iter()
                            .filter(|e| e.is_required)
                            .map(|e| e.options
                                 .as_ref()
                                 .and_then(|opts| opts.first())
                                 .map(|first| (e.name.to_owned(), first.to_owned()))
                            )
                            .collect::<Option<Vec<ExtraProp>>>()?;
                        let load = ResourceRequest {
                            base: a.transport_url.to_owned(),
                            path: ResourceRef::with_extra("catalog", &cat.type_name, &cat.id, &props)
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
                    .unique_by(|cat_entry| &cat_entry.load.path.type_name)
                    .map(|cat_entry| TypeEntry {
                        is_selected: resource_req.path.type_name == cat_entry.load.path.type_name,
                        type_name: cat_entry.load.path.type_name.to_owned(),
                        load: cat_entry.load.to_owned()
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
