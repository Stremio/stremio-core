use crate::constants::{CATALOG_RESOURCE_NAME, SKIP_EXTRA_PROP};
use crate::models::common::{
    eq_update, resource_update_with_vector_content, Loadable, ResourceAction, ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::{
    AggrRequest, Descriptor, ExtraExt, ExtraValue, ManifestCatalog, ResourcePath, ResourceRequest,
};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use futures::FutureExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::ops::Range;

pub const SPOTLIGHT_ITEMS_COUNT: usize = 6;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub r#type: Option<String>,
    #[serde(default)]
    pub extra: Vec<ExtraValue>,
}

pub type CatalogPage<T> = ResourceLoadable<Vec<T>>;

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
                let spotlight_effects = self
                    .spotlight
                    .as_mut()
                    .filter(|page| page.request == *request)
                    .map(|page| {
                        resource_update_with_vector_content::<E, _>(
                            page,
                            ResourceAction::ResourceRequestResult { request, result },
                        )
                    })
                    .unwrap_or_else(|| Effects::none().unchanged());
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

pub fn board_catalogs(
    addons: &[Descriptor],
) -> Vec<(&Descriptor, &ManifestCatalog, ResourceRequest)> {
    addons
        .iter()
        .flat_map(|addon| {
            addon
                .manifest
                .catalogs
                .iter()
                .filter(|catalog| catalog.is_board_catalog())
                .map(move |catalog| {
                    (
                        addon,
                        catalog,
                        ResourceRequest::new(
                            addon.transport_url.to_owned(),
                            ResourcePath::without_extra(
                                CATALOG_RESOURCE_NAME,
                                &catalog.r#type,
                                &catalog.id,
                            ),
                        ),
                    )
                })
        })
        .collect()
}

pub fn board_spotlight_request(profile: &Profile) -> Option<ResourceRequest> {
    let catalogs = board_catalogs(&profile.addons);
    profile
        .settings
        .board_spotlight
        .catalog
        .as_ref()
        .filter(|catalog| catalogs.iter().any(|(_, _, request)| request == *catalog))
        .cloned()
        .or_else(|| catalogs.into_iter().next().map(|(_, _, request)| request))
}

fn spotlight_request(selected: &Option<Selected>, profile: &Profile) -> Option<ResourceRequest> {
    match selected {
        Some(Selected { r#type, extra })
            if r#type.is_none() && extra.is_empty() && profile.settings.board_spotlight.enabled =>
        {
            board_spotlight_request(profile)
        }
        _ => None,
    }
}

pub fn spotlight_items(items: &[MetaItemPreview]) -> impl Iterator<Item = &MetaItemPreview> {
    items
        .iter()
        .filter(|item| item.background.is_some())
        .unique_by(|item| &item.id)
        .take(SPOTLIGHT_ITEMS_COUNT)
}

pub fn resource_effect<E: Env + 'static>(request: ResourceRequest) -> Effect {
    EffectFuture::Concurrent(
        E::addon_transport(&request.base)
            .resource(&request.path)
            .map(|result| Msg::Internal(Internal::ResourceRequestResult(request, Box::new(result))))
            .boxed_env(),
    )
    .into()
}

fn catalogs_update<E: Env + 'static>(
    catalogs: &mut Vec<Catalog<MetaItemPreview>>,
    spotlight: &mut Option<CatalogPage<MetaItemPreview>>,
    selected: &Option<Selected>,
    range: Option<&Range<usize>>,
    profile: &Profile,
) -> Effects {
    let spotlight_request = spotlight_request(selected, profile);
    let (next_catalogs, effects) = match selected {
        Some(selected) => {
            let request = AggrRequest::AllCatalogs {
                extra: &selected.extra,
                r#type: &selected.r#type,
            };
            request
                .plan(&profile.addons)
                .into_iter()
                .map(|(_, request)| request)
                .filter(|request| {
                    profile.settings.board_spotlight.show_catalog_row
                        || spotlight_request.as_ref() != Some(request)
                })
                .enumerate()
                .map(|(index, request)| {
                    catalogs
                        .iter()
                        .find(|catalog| {
                            matches!(catalog.first(), Some(resource) if resource.request == request && resource.content.is_some())
                        })
                        .map(|catalog| (catalog.to_owned(), None))
                        .or_else(|| {
                            spotlight
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
    let (next_spotlight, spotlight_effect) = match spotlight_request {
        Some(request) => spotlight
            .as_ref()
            .filter(|page| page.request == request && page.content.is_some())
            .or_else(|| {
                next_catalogs.iter().find_map(|catalog| {
                    catalog
                        .first()
                        .filter(|page| page.request == request && page.content.is_some())
                })
            })
            .map(|page| (Some(page.to_owned()), None))
            .unwrap_or_else(|| {
                (
                    Some(ResourceLoadable {
                        request: request.to_owned(),
                        content: Some(Loadable::Loading),
                    }),
                    Some(resource_effect::<E>(request)),
                )
            }),
        None => (None, None),
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
