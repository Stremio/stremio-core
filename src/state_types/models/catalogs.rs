use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::*;
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
const PAGE_LEN: u32 = 100;
const SKIP: &str = "skip";

#[derive(Serialize, Clone, Debug)]
pub struct TypeEntry {
    pub is_selected: bool,
    pub type_name: String,
    pub load: ResourceRequest,
}

#[derive(Serialize, Clone, Debug)]
pub struct CatalogEntry {
    pub is_selected: bool,
    pub name: String,
    pub load: ResourceRequest,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CatalogFiltered {
    pub types: Vec<TypeEntry>,
    pub catalogs: Vec<CatalogEntry>,
    // selectable_extra are the extra props the user can select from (e.g. Genre, Year)
    // selectable_extra does not have a .load property - cause to a large extent,
    // the UI is responsible for that logic: whether it's gonna allow selecting multiple options of
    // one prop, and/or allow combining extra props
    pub selectable_extra: Vec<ManifestExtraProp>,
    pub selected: Option<ResourceRequest>,
    // @TODO more sophisticated error, such as EmptyContent/UninstalledAddon/Offline
    // see https://github.com/Stremio/stremio/issues/402
    pub content: Loadable<Vec<MetaPreview>, String>,
    // Pagination: loading previous/next pages
    pub load_next: Option<ResourceRequest>,
    pub load_prev: Option<ResourceRequest>,
    // NOTE: There's no currently selected preview item, cause some UIs may not have this
    // so, it should be implemented in the UI
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered(selected_req))) => {
                let addons = &ctx.content.addons;
                // Catalogs are NOT filtered by type, cause the UI gets to decide whether to
                // only show catalogs for the selected type, or all of them
                let catalogs: Vec<CatalogEntry> = addons
                    .iter()
                    .flat_map(|a| {
                        a.manifest.catalogs.iter().filter_map(move |cat| {
                            // Required properties are allowed, but only if there's .options
                            // with at least one option inside (that we default to)
                            // If there are no required properties at all, this will resolve to Some([])
                            let props = cat
                                .extra_iter()
                                .filter(|e| e.is_required)
                                .map(|e| {
                                    e.options
                                        .as_ref()
                                        .and_then(|opts| opts.first())
                                        .map(|first| (e.name.to_owned(), first.to_owned()))
                                })
                                .collect::<Option<Vec<ExtraProp>>>()?;
                            let load = ResourceRequest {
                                base: a.transport_url.to_owned(),
                                path: ResourceRef::with_extra(
                                    "catalog",
                                    &cat.type_name,
                                    &cat.id,
                                    &props,
                                ),
                            };
                            Some(CatalogEntry {
                                name: cat.name.as_ref().unwrap_or(&a.manifest.name).to_owned(),
                                is_selected: load.eq_no_extra(selected_req),
                                load,
                            })
                        })
                    })
                    .collect();
                // We are using unique_by in order to preserve the original order
                // in which the types appear in
                let types = catalogs
                    .iter()
                    .unique_by(|cat_entry| &cat_entry.load.path.type_name)
                    .map(|cat_entry| TypeEntry {
                        is_selected: selected_req.path.type_name == cat_entry.load.path.type_name,
                        type_name: cat_entry.load.path.type_name.to_owned(),
                        load: cat_entry.load.to_owned(),
                    })
                    .collect();
                // Find the selected catalog, and get it's extra_iter
                let selectable_extra = addons
                    .iter()
                    .find(|a| a.transport_url == selected_req.base)
                    .iter()
                    .flat_map(|a| &a.manifest.catalogs)
                    .find(|cat| cat.type_name == selected_req.path.type_name && cat.id == selected_req.path.id)
                    .map(|cat| cat.extra_iter().map(|x| x.into_owned()).collect::<Vec<_>>())
                    .unwrap_or_default();
                // Reset the model state
                // content will be Loadable::Loading
                *self = CatalogFiltered {
                    catalogs,
                    types,
                    selectable_extra,
                    selected: Some(selected_req.to_owned()),
                    ..Default::default()
                };
                Effects::one(addon_get::<Env>(&selected_req))
            }
            Msg::Internal(AddonResponse(req, result))
                if Some(req) == self.selected.as_ref() && self.content == Loadable::Loading =>
            {
                self.content = match result.as_ref() {
                    Ok(ResourceResponse::Metas { metas }) => {
                        let skip = get_skip(&req.path);
                        self.load_prev = match skip {
                            i if i >= PAGE_LEN && i % PAGE_LEN == 0 => {
                                Some(with_skip(req, i - PAGE_LEN))
                            }
                            _ => None,
                        };
                        // If we return more, we still shouldn't allow a next page,
                        // because we're only ever rendering PAGE_LEN at a time
                        self.load_next = match metas.len() as u32 {
                            PAGE_LEN => Some(with_skip(req, skip + PAGE_LEN)),
                            _ => None,
                        };
                        Loadable::Ready(metas.iter().take(PAGE_LEN as usize).cloned().collect())
                    }
                    Ok(_) => Loadable::Err(UNEXPECTED_RESP_MSG.to_owned()),
                    Err(e) => Loadable::Err(e.to_string()),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_skip(path: &ResourceRef) -> u32 {
    path.get_extra_first_val(SKIP)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
}

fn with_skip(req: &ResourceRequest, skip: u32) -> ResourceRequest {
    let mut req = req.to_owned();
    req.path.set_extra_unique(SKIP, skip.to_string());
    req
}
