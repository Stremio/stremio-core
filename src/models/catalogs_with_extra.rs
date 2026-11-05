use crate::constants::SKIP_EXTRA_PROP;
use crate::models::common::{
    board_plan, board_spotlight_request, eq_update, resource_effect,
    resource_update_with_vector_content, spotlight_items, Loadable, ResourceAction,
    ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad, Internal, Msg};
use crate::runtime::{EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ExtraExt, ExtraValue, ResourcePath, ResourceRequest};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::ops::Range;

pub use crate::models::common::CatalogPage;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub r#type: Option<String>,
    #[serde(default)]
    pub extra: Vec<ExtraValue>,
}

pub type Catalog<T> = Vec<CatalogPage<T>>;

#[derive(Default, Clone, Serialize, Debug)]
pub struct CatalogsWithExtra {
    pub selected: Option<Selected>,
    pub catalogs: Vec<Catalog<MetaItemPreview>>,
    pub spotlight: Option<CatalogPage<MetaItemPreview>>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for CatalogsWithExtra {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(selected))) => {
                let selected_effects = selected_update(&mut self.selected, selected);
                let catalogs_effects = catalogs_update::<E>(
                    &mut self.catalogs,
                    &mut self.spotlight,
                    &self.selected,
                    None,
                    &ctx.profile,
                );
                let search_effects = match &self.selected {
                    Some(Selected { extra, .. }) => match extra
                        .iter()
                        .find(|ExtraValue { name, .. }| name == "search")
                    {
                        Some(ExtraValue { value, .. }) => {
                            Effects::msg(Msg::Internal(Internal::CatalogsWithExtraSearch {
                                query: value.to_owned(),
                            }))
                            .unchanged()
                        }
                        None => Effects::none().unchanged(),
                    },
                    None => Effects::none().unchanged(),
                };
                selected_effects.join(catalogs_effects).join(search_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalogs_effects = eq_update(&mut self.catalogs, vec![]);
                let spotlight_effects = eq_update(&mut self.spotlight, None);
                selected_effects
                    .join(catalogs_effects)
                    .join(spotlight_effects)
            }
            Msg::Action(Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadRange(range))) => {
                catalogs_update::<E>(
                    &mut self.catalogs,
                    &mut self.spotlight,
                    &self.selected,
                    Some(range),
                    &ctx.profile,
                )
            }
            Msg::Action(Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadNextPage(
                index,
            ))) => match self.catalogs.get_mut(*index) {
                Some(catalog) => match catalog.last() {
                    Some(ResourceLoadable {
                        content: Some(Loadable::Ready(items)),
                        request,
                    }) if ctx
                        .profile
                        .addons
                        .iter()
                        .find(|addon| addon.transport_url == request.base)
                        .and_then(|addon| {
                            addon.manifest.catalogs.iter().find(|manifest_catalog| {
                                manifest_catalog.id == request.path.id
                                    && manifest_catalog.r#type == request.path.r#type
                            })
                        })
                        .map(|manifest_catalog| {
                            manifest_catalog
                                .extra
                                .iter()
                                .any(|extra_prop| extra_prop.name == SKIP_EXTRA_PROP.name)
                        })
                        .unwrap_or_default() =>
                    {
                        let skip = request
                            .path
                            .extra
                            .iter()
                            .find(|extra_prop| extra_prop.name == SKIP_EXTRA_PROP.name)
                            .and_then(|extra_prop| extra_prop.value.parse::<usize>().ok())
                            .unwrap_or_default();
                        let skip = skip + items.len();
                        let request = ResourceRequest {
                            base: request.base.to_owned(),
                            path: ResourcePath {
                                id: request.path.id.to_owned(),
                                r#type: request.path.r#type.to_owned(),
                                resource: request.path.resource.to_owned(),
                                extra: request
                                    .path
                                    .extra
                                    .to_owned()
                                    .extend_one(&SKIP_EXTRA_PROP, Some(skip.to_string())),
                            },
                        };
                        catalog.push(ResourceLoadable {
                            request: request.to_owned(),
                            content: Some(Loadable::Loading),
                        });
                        Effects::one(
                            EffectFuture::Concurrent(
                                E::addon_transport(&request.base)
                                    .resource(&request.path)
                                    .map(|result| {
                                        Msg::Internal(Internal::ResourceRequestResult(
                                            request,
                                            Box::new(result),
                                        ))
                                    })
                                    .boxed_env(),
                            )
                            .into(),
                        )
                    }
                    _ => Effects::none().unchanged(),
                },
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let catalogs_effects = self
                    .catalogs
                    .iter_mut()
                    .find_map(|catalog| catalog.last_mut().filter(|page| page.request == *request))
                    .map(|page| {
                        resource_update_with_vector_content::<E, _>(
                            page,
                            ResourceAction::ResourceRequestResult { request, result },
                        )
                    })
                    .unwrap_or_else(|| Effects::none().unchanged());
                let spotlight_effects = match self
                    .spotlight
                    .as_mut()
                    .filter(|page| page.request == *request)
                {
                    Some(page) => resource_update_with_vector_content::<E, _>(
                        page,
                        ResourceAction::ResourceRequestResult { request, result },
                    )
                    .join(catalogs_update::<E>(
                        &mut self.catalogs,
                        &mut self.spotlight,
                        &self.selected,
                        None,
                        &ctx.profile,
                    )),
                    None => Effects::none().unchanged(),
                };
                catalogs_effects.join(spotlight_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => catalogs_update::<E>(
                &mut self.catalogs,
                &mut self.spotlight,
                &self.selected,
                None,
                &ctx.profile,
            ),
            Msg::Internal(Internal::LibraryChanged(_)) => Effects::none(),
            _ => Effects::none().unchanged(),
        }
    }
}

fn selected_update(selected: &mut Option<Selected>, next_selected: &Selected) -> Effects {
    let mut next_selected = next_selected.to_owned();
    next_selected.extra = next_selected.extra.remove_all(&SKIP_EXTRA_PROP);
    eq_update(selected, Some(next_selected))
}

fn is_board(selected: &Selected) -> bool {
    selected.r#type.is_none() && selected.extra.is_empty()
}

fn spotlight_request(selected: &Option<Selected>, profile: &Profile) -> Option<ResourceRequest> {
    match selected {
        Some(selected) if is_board(selected) && profile.settings.board_spotlight.enabled => {
            board_spotlight_request(profile)
        }
        _ => None,
    }
}

fn spotlight_hides_row(spotlight: &Option<CatalogPage<MetaItemPreview>>) -> bool {
    match spotlight.as_ref().and_then(|page| page.content.as_ref()) {
        Some(Loadable::Ready(items)) => spotlight_items(items).next().is_some(),
        Some(Loadable::Err(_)) => false,
        _ => true,
    }
}

fn catalogs_update<E: Env + 'static>(
    catalogs: &mut Vec<Catalog<MetaItemPreview>>,
    spotlight: &mut Option<CatalogPage<MetaItemPreview>>,
    selected: &Option<Selected>,
    range: Option<&Range<usize>>,
    profile: &Profile,
) -> Effects {
    let spotlight_request = spotlight_request(selected, profile);
    let (next_spotlight, spotlight_effect) = match &spotlight_request {
        Some(request) => spotlight
            .as_ref()
            .filter(|page| page.request == *request && page.content.is_some())
            .or_else(|| {
                catalogs.iter().find_map(|catalog| {
                    catalog
                        .first()
                        .filter(|page| page.request == *request && page.content.is_some())
                })
            })
            .map(|page| (Some(page.to_owned()), None))
            .unwrap_or_else(|| {
                (
                    Some(ResourceLoadable {
                        request: request.to_owned(),
                        content: Some(Loadable::Loading),
                    }),
                    Some(resource_effect::<E>(request.to_owned())),
                )
            }),
        None => (None, None),
    };
    let hidden_request = spotlight_request
        .filter(|_| !profile.settings.board_spotlight.show_catalog_row)
        .filter(|_| spotlight_hides_row(&next_spotlight));
    let (next_catalogs, effects) = match selected {
        Some(selected) => {
            let requests = if is_board(selected) {
                board_plan(profile)
                    .into_iter()
                    .map(|(_, _, request)| request)
                    .collect::<Vec<_>>()
            } else {
                AggrRequest::AllCatalogs {
                    extra: &selected.extra,
                    r#type: &selected.r#type,
                }
                .plan(&profile.addons)
                .into_iter()
                .map(|(_, request)| request)
                .collect::<Vec<_>>()
            };
            requests
                .into_iter()
                .filter(|request| hidden_request.as_ref() != Some(request))
                .enumerate()
                .map(|(index, request)| {
                    catalogs
                        .iter()
                        .find(|catalog| {
                            matches!(catalog.first(), Some(resource) if resource.request == request && resource.content.is_some())
                        })
                        .map(|catalog| (catalog.to_owned(), None))
                        .or_else(|| {
                            next_spotlight
                                .as_ref()
                                .filter(|page| page.request == request && page.content.is_some())
                                .map(|page| (vec![page.to_owned()], None))
                        })
                        .unwrap_or_else(|| match range {
                            Some(range) if range.start <= index && index <= range.end => (
                                vec![ResourceLoadable {
                                    request: request.to_owned(),
                                    content: Some(Loadable::Loading),
                                }],
                                Some(resource_effect::<E>(request)),
                            ),
                            _ => (
                                vec![ResourceLoadable {
                                    request,
                                    content: None,
                                }],
                                None,
                            ),
                        })
                })
                .unzip::<_, _, Vec<_>, Vec<_>>()
        }
        _ => Default::default(),
    };
    Effects::many(
        effects
            .into_iter()
            .chain(std::iter::once(spotlight_effect))
            .flatten()
            .collect(),
    )
    .unchanged()
    .join(eq_update(catalogs, next_catalogs))
    .join(eq_update(spotlight, next_spotlight))
}
