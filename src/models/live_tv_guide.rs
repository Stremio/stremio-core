use chrono::{Days, NaiveDate};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{CATALOG_RESOURCE_NAME, EPG_DATE_EXTRA_PROP, SKIP_EXTRA_PROP},
    models::{
        common::{
            eq_update, resource_update_with_vector_content, ResourceAction, ResourceLoadable,
        },
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLiveTvGuide, ActionLoad, Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        addon::{ExtraExt, ResourcePath, ResourceRequest},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video},
    },
};

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    /// Guide catalog request (without the `date` extra);
    /// when `None`, the first guide catalog of the installed
    /// `epgProvider` addons is used
    pub request: Option<ResourceRequest>,
    /// Guide date; when `None`, defaults to today
    pub date: Option<NaiveDate>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectableCatalog {
    /// Catalog name (or its id, when it has no name)
    pub catalog: String,
    pub addon_name: String,
    pub selected: bool,
    /// Guide catalog request (without the `date` extra)
    pub request: ResourceRequest,
}

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
pub struct SelectablePage {
    pub request: ResourceRequest,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selectable {
    pub catalogs: Vec<SelectableCatalog>,
    pub prev_date: Option<NaiveDate>,
    pub next_date: Option<NaiveDate>,
    pub today: Option<NaiveDate>,
    /// The next channels page request (with the `date` and `skip` extras);
    /// present only when the selected catalog declares the `skip` extra
    /// and all requested pages are loaded
    pub next_page: Option<SelectablePage>,
}

/// A channel with its program for the selected date,
/// shows ordered by start time
#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelGuide {
    pub channel: MetaItemPreview,
    pub shows: Vec<Video>,
}

pub type CatalogPage = ResourceLoadable<Vec<MetaItem>>;

pub type Catalog = Vec<CatalogPage>;

enum CatalogPageRequest {
    First,
    Next,
}

/// The program guide grid (channels x shows) of live TV channels
/// provided by addons with the `epgProvider` manifest behavior hint
#[derive(Derivative, Clone, Serialize, Debug)]
#[derivative(Default(bound = ""))]
pub struct LiveTvGuide {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Catalog,
    pub channels: Vec<ChannelGuide>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for LiveTvGuide {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LiveTvGuide(selected))) => {
                let selected_effects =
                    selected_update::<E>(&mut self.selected, selected, &ctx.profile);
                let catalog_effects = match self.selected.as_ref() {
                    Some(Selected {
                        request: Some(request),
                        date: Some(date),
                    }) => catalog_update::<E>(
                        &mut self.catalog,
                        CatalogPageRequest::First,
                        &with_date_extra(request, date),
                    ),
                    _ => eq_update(&mut self.catalog, vec![]),
                };
                let selectable_effects = selectable_update::<E>(
                    &mut self.selectable,
                    &self.selected,
                    &self.catalog,
                    &ctx.profile,
                );
                let channels_effects =
                    channels_update(&mut self.channels, &self.selected, &self.catalog);

                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
                    .join(channels_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects = eq_update(&mut self.selectable, Selectable::default());
                let catalog_effects = eq_update(&mut self.catalog, vec![]);
                let channels_effects = eq_update(&mut self.channels, Vec::new());

                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
                    .join(channels_effects)
            }
            Msg::Action(Action::LiveTvGuide(ActionLiveTvGuide::LoadNextPage)) => {
                match self.selectable.next_page.as_ref() {
                    Some(next_page) => {
                        let catalog_effects = catalog_update::<E>(
                            &mut self.catalog,
                            CatalogPageRequest::Next,
                            &next_page.request,
                        );
                        let selectable_effects = selectable_update::<E>(
                            &mut self.selectable,
                            &self.selected,
                            &self.catalog,
                            &ctx.profile,
                        );

                        catalog_effects.join(selectable_effects)
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => self
                .catalog
                .iter_mut()
                .find(|page| page.request == *request)
                .map(|page| {
                    resource_update_with_vector_content::<E, MetaItem>(
                        page,
                        ResourceAction::ResourceRequestResult { request, result },
                    )
                })
                .map(|catalog_effects| {
                    let selectable_effects = selectable_update::<E>(
                        &mut self.selectable,
                        &self.selected,
                        &self.catalog,
                        &ctx.profile,
                    );
                    let channels_effects =
                        channels_update(&mut self.channels, &self.selected, &self.catalog);

                    catalog_effects
                        .join(selectable_effects)
                        .join(channels_effects)
                })
                .unwrap_or_else(|| Effects::none().unchanged()),
            Msg::Internal(Internal::ProfileChanged) => selectable_update::<E>(
                &mut self.selectable,
                &self.selected,
                &self.catalog,
                &ctx.profile,
            ),
            _ => Effects::none().unchanged(),
        }
    }
}

fn selected_update<E: Env + 'static>(
    selected: &mut Option<Selected>,
    next_selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    let request = next_selected
        .as_ref()
        .and_then(|selected| selected.request.to_owned())
        .or_else(|| {
            guide_catalogs(profile)
                .next()
                .map(|(_, _, request)| request)
        });
    let date = next_selected
        .as_ref()
        .and_then(|selected| selected.date)
        .or_else(|| Some(E::now().date_naive()));

    eq_update(selected, Some(Selected { request, date }))
}

fn catalog_update<E: Env + 'static>(
    catalog: &mut Catalog,
    page_request: CatalogPageRequest,
    request: &ResourceRequest,
) -> Effects {
    let mut page = ResourceLoadable {
        request: request.to_owned(),
        content: None,
    };
    let effects = resource_update_with_vector_content::<E, MetaItem>(
        &mut page,
        ResourceAction::ResourceRequested { request },
    );
    match page_request {
        CatalogPageRequest::First => *catalog = vec![page],
        CatalogPageRequest::Next => catalog.extend(vec![page]),
    };

    effects
}

fn selectable_update<E: Env + 'static>(
    selectable: &mut Selectable,
    selected: &Option<Selected>,
    catalog: &Catalog,
    profile: &Profile,
) -> Effects {
    let catalogs = guide_catalogs(profile)
        .map(|(addon_name, catalog_name, request)| SelectableCatalog {
            catalog: catalog_name,
            addon_name,
            selected: selected
                .as_ref()
                .and_then(|selected| selected.request.as_ref())
                .map(|selected_request| selected_request.eq_no_extra(&request))
                .unwrap_or_default(),
            request,
        })
        .collect::<Vec<_>>();
    let date = selected.as_ref().and_then(|selected| selected.date);
    let next_page = next_page_update(selected, catalog, profile);
    let updated_selectable = Selectable {
        catalogs,
        prev_date: date.and_then(|date| date.checked_sub_days(Days::new(1))),
        next_date: date.and_then(|date| date.checked_add_days(Days::new(1))),
        today: Some(E::now().date_naive()),
        next_page,
    };

    eq_update(selectable, updated_selectable)
}

fn next_page_update(
    selected: &Option<Selected>,
    catalog: &Catalog,
    profile: &Profile,
) -> Option<SelectablePage> {
    let (request, date) = match selected {
        Some(Selected {
            request: Some(request),
            date: Some(date),
        }) => (request, date),
        _ => return None,
    };

    profile
        .addons
        .iter()
        .filter(|addon| addon.manifest.behavior_hints.epg_provider)
        .find(|addon| addon.transport_url == request.base)
        .and_then(|addon| {
            addon.manifest.catalogs.iter().find(|manifest_catalog| {
                manifest_catalog.id == request.path.id
                    && manifest_catalog.r#type == request.path.r#type
            })
        })
        // unlike the `date` extra, pagination is opt-in:
        // the catalog has to declare the `skip` extra
        .filter(|manifest_catalog| {
            manifest_catalog
                .extra
                .iter()
                .any(|extra_prop| extra_prop.name == SKIP_EXTRA_PROP.name)
        })
        .and_then(|_| {
            catalog
                .iter()
                .map(|page| {
                    page.content
                        .as_ref()
                        .and_then(|content| content.ready())
                        .filter(|content| !content.is_empty())
                        .map(|content| content.len())
                })
                .collect::<Option<Vec<_>>>()
                .map(|page_sizes| page_sizes.into_iter().sum::<usize>())
        })
        .map(|skip| {
            let page_request = with_date_extra(request, date);
            SelectablePage {
                request: ResourceRequest {
                    base: page_request.base.to_owned(),
                    path: ResourcePath {
                        extra: page_request
                            .path
                            .extra
                            .to_owned()
                            .extend_one(&SKIP_EXTRA_PROP, Some(skip.to_string())),
                        ..page_request.path
                    },
                },
            }
        })
}

fn channels_update(
    channels: &mut Vec<ChannelGuide>,
    selected: &Option<Selected>,
    catalog: &Catalog,
) -> Effects {
    let updated_channels = match selected {
        Some(Selected {
            date: Some(date), ..
        }) => {
            let day_start = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();
            let day_end = date
                .checked_add_days(Days::new(1))
                .and_then(|next_date| next_date.and_hms_opt(0, 0, 0))
                .unwrap_or_default()
                .and_utc();

            catalog
                .iter()
                .filter_map(|page| page.content.as_ref().and_then(|content| content.ready()))
                .flatten()
                .unique_by(|meta_item| meta_item.preview.id.to_owned())
                .map(|meta_item| ChannelGuide {
                    channel: meta_item.preview.to_owned(),
                    shows: meta_item
                        .videos
                        .iter()
                        // include only program shows overlapping the selected date
                        .filter(|video| {
                            video.epg_info.as_ref().is_some_and(|epg_info| {
                                epg_info.start_time < day_end && epg_info.end_time > day_start
                            })
                        })
                        .sorted_by_key(|video| {
                            video.epg_info.as_ref().map(|epg_info| epg_info.start_time)
                        })
                        .cloned()
                        .collect(),
                })
                .collect()
        }
        _ => vec![],
    };

    eq_update(channels, updated_channels)
}

/// Guide catalogs of the installed `epgProvider` addons as
/// `(addon_name, catalog_name, request)`, requests without the `date` extra
fn guide_catalogs(
    profile: &Profile,
) -> impl Iterator<Item = (String, String, ResourceRequest)> + '_ {
    profile
        .addons
        .iter()
        .filter(|addon| addon.manifest.behavior_hints.epg_provider)
        .flat_map(|addon| {
            addon
                .manifest
                .catalogs
                .iter()
                .filter_map(move |manifest_catalog| {
                    manifest_catalog.default_required_extra().map(|extra| {
                        (
                            addon.manifest.name.to_owned(),
                            manifest_catalog
                                .name
                                .as_ref()
                                .unwrap_or(&manifest_catalog.id)
                                .to_owned(),
                            ResourceRequest {
                                base: addon.transport_url.to_owned(),
                                path: ResourcePath {
                                    resource: CATALOG_RESOURCE_NAME.to_owned(),
                                    r#type: manifest_catalog.r#type.to_owned(),
                                    id: manifest_catalog.id.to_owned(),
                                    extra,
                                },
                            },
                        )
                    })
                })
        })
}

/// The guide request for the given date: the `date` extra is appended
/// unconditionally — `epgProvider` addons support it by contract,
/// whether or not their catalogs declare it
fn with_date_extra(request: &ResourceRequest, date: &NaiveDate) -> ResourceRequest {
    ResourceRequest {
        base: request.base.to_owned(),
        path: ResourcePath {
            extra: request.path.extra.to_owned().extend_one(
                &EPG_DATE_EXTRA_PROP,
                Some(date.format("%Y-%m-%d").to_string()),
            ),
            ..request.path.to_owned()
        },
    }
}
