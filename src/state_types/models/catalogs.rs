use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::*;
use crate::types::MetaPreview;
use itertools::*;
use serde_derive::*;
use std::convert::TryFrom;
use derivative::*;

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

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum CatalogError {
    EmptyContent,
    UnexpectedResp,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Derivative)]
#[derivative(Default(bound=""))]
pub struct CatalogFiltered<T> {
    pub types: Vec<TypeEntry>,
    pub catalogs: Vec<CatalogEntry>,
    // selectable_extra are the extra props the user can select from (e.g. Genre, Year)
    // selectable_extra does not have a .load property - cause to a large extent,
    // the UI is responsible for that logic: whether it's gonna allow selecting multiple options of
    // one prop, and/or allow combining extra props
    // Usage (UI) guide:
    // * Be careful whether the property `is_required`; if it's not, you can show a "None" option
    // * the default `.load` for the given catalog will always pass a default for a given extra
    // prop if it `is_required`
    // * to check if it's selected, you just need to find a corresponding key/value pair in .selected.path.extra
    // * keep in mind, many may be selected, if you want to allow that in the UI
    // * in this case, you must comply to options_limit
    pub selectable_extra: Vec<ManifestExtraProp>,
    pub selected: Option<ResourceRequest>,
    pub content: Loadable<Vec<T>, CatalogError>,
    // Pagination: loading previous/next pages
    pub load_next: Option<ResourceRequest>,
    pub load_prev: Option<ResourceRequest>,
    // NOTE: There's no currently selected preview item, cause some UIs may not have this
    // so, it should be implemented in the UI
}

impl<Env, T> UpdateWithCtx<Ctx<Env>> for CatalogFiltered<T>
where
    Env: Environment + 'static,
    T: PartialEq,
    Vec<T>: TryFrom<ResourceResponse>,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        let addons = &ctx.content.addons;
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered(selected_req))) => {
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
                                        // @TODO .or_else to fill from defaults that the
                                        // CatalogFiltered was constructed with
                                        // although it won't work here, cause we need to filter
                                        // such that every catalog has the prop
                                        .map(|first| (e.name.to_owned(), first.to_owned()))
                                })
                                // .collect will return None if at least one of the items in the
                                // iterator is None
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
                let selectable_extra = get_catalog(addons, &selected_req)
                    .map(|cat| {
                        cat.extra_iter()
                            .filter(|x| x.options.iter().flatten().next().is_some())
                            .map(|x| x.into_owned())
                            .collect::<Vec<_>>()
                    })
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
            Msg::Internal(AddonResponse(req, resp))
                if Some(req) == self.selected.as_ref() && self.content == Loadable::Loading =>
            {
                let skippable = get_catalog(addons, &req)
                    .map(|cat| cat.extra_iter().any(|e| e.name == SKIP))
                    .unwrap_or(false);
                let len = match resp.as_ref() {
                    Ok(ResourceResponse::Metas { metas }) => metas.len() as u32,
                    _ => 0,
                };
                let skip = get_skip(&req.path);

                // Set .load_prev/load_next, which are direct references to the prev/next page
                self.load_prev = if skippable && skip >= PAGE_LEN && skip % PAGE_LEN == 0 {
                    Some(with_skip(req, skip - PAGE_LEN))
                } else {
                    None
                };
                // If we return more, we still shouldn't allow a next page,
                // because we're only ever rendering PAGE_LEN at a time
                self.load_next = if skippable && len == PAGE_LEN {
                    Some(with_skip(req, skip + PAGE_LEN))
                } else {
                    None
                };

                self.content = match resp.as_ref() {
                    Ok(resp) => match <Vec<T>>::try_from(resp.to_owned()) {
                        Ok(ref x) if x.is_empty() => Loadable::Err(CatalogError::EmptyContent),
                        Ok(x) => Loadable::Ready(x.into_iter().take(PAGE_LEN as usize).collect()),
                        Err(_) => Loadable::Err(CatalogError::UnexpectedResp),
                    },
                    Err(e) => Loadable::Err(CatalogError::Other(e.to_string())),
                };
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn get_catalog<'a>(addons: &'a [Descriptor], req: &ResourceRequest) -> Option<&'a ManifestCatalog> {
    addons
        .iter()
        .find(|a| a.transport_url == req.base)
        .iter()
        .flat_map(|a| &a.manifest.catalogs)
        .find(|cat| cat.type_name == req.path.type_name && cat.id == req.path.id)
}

fn get_skip(path: &ResourceRef) -> u32 {
    path.get_extra_first_val(SKIP)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn with_skip(req: &ResourceRequest, skip: u32) -> ResourceRequest {
    let mut req = req.to_owned();
    req.path.set_extra_unique(SKIP, skip.to_string());
    req
}
