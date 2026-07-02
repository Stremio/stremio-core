use chrono::{Days, NaiveDate};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{CATALOG_RESOURCE_NAME, EPG_DATE_EXTRA_PROP},
    models::{
        common::{
            eq_update, resource_update_with_vector_content, Loadable, ResourceAction,
            ResourceLoadable,
        },
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
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

#[derive(Default, Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selectable {
    pub catalogs: Vec<SelectableCatalog>,
    pub prev_date: Option<NaiveDate>,
    pub next_date: Option<NaiveDate>,
    pub today: Option<NaiveDate>,
}

/// A channel with its program for the selected date,
/// shows ordered by start time
#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelGuide {
    pub channel: MetaItemPreview,
    pub shows: Vec<Video>,
}

/// The program guide grid (channels x shows) of live TV channels
/// provided by addons with the `epgProvider` manifest behavior hint
#[derive(Derivative, Clone, Serialize, Debug)]
#[derivative(Default(bound = ""))]
pub struct LiveTvGuide {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Option<ResourceLoadable<Vec<MetaItem>>>,
    pub channels: Vec<ChannelGuide>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for LiveTvGuide {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LiveTvGuide(selected))) => {
                let selected_effects =
                    selected_update::<E>(&mut self.selected, selected, &ctx.profile);
                let catalog_effects = catalog_update::<E>(&mut self.catalog, &self.selected);
                let selectable_effects =
                    selectable_update::<E>(&mut self.selectable, &self.selected, &ctx.profile);
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
                let catalog_effects = eq_update(&mut self.catalog, None);
                let channels_effects = eq_update(&mut self.channels, Vec::new());

                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
                    .join(channels_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let catalog_effects = match &mut self.catalog {
                    Some(catalog) => resource_update_with_vector_content::<E, MetaItem>(
                        catalog,
                        ResourceAction::ResourceRequestResult { request, result },
                    ),
                    None => Effects::none().unchanged(),
                };
                let channels_effects = if catalog_effects.has_changed {
                    channels_update(&mut self.channels, &self.selected, &self.catalog)
                } else {
                    Effects::none().unchanged()
                };

                catalog_effects.join(channels_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                selectable_update::<E>(&mut self.selectable, &self.selected, &ctx.profile)
            }
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
    catalog: &mut Option<ResourceLoadable<Vec<MetaItem>>>,
    selected: &Option<Selected>,
) -> Effects {
    match selected {
        Some(Selected {
            request: Some(request),
            date: Some(date),
        }) => {
            let request = with_date_extra(request, date);
            let catalog = catalog.get_or_insert_with(|| ResourceLoadable {
                request: request.to_owned(),
                content: None,
            });

            resource_update_with_vector_content::<E, MetaItem>(
                catalog,
                ResourceAction::ResourceRequested { request: &request },
            )
        }
        _ => eq_update(catalog, None),
    }
}

fn selectable_update<E: Env + 'static>(
    selectable: &mut Selectable,
    selected: &Option<Selected>,
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
    let updated_selectable = Selectable {
        catalogs,
        prev_date: date.and_then(|date| date.checked_sub_days(Days::new(1))),
        next_date: date.and_then(|date| date.checked_add_days(Days::new(1))),
        today: Some(E::now().date_naive()),
    };

    eq_update(selectable, updated_selectable)
}

fn channels_update(
    channels: &mut Vec<ChannelGuide>,
    selected: &Option<Selected>,
    catalog: &Option<ResourceLoadable<Vec<MetaItem>>>,
) -> Effects {
    let updated_channels = match (selected, catalog) {
        (
            Some(Selected {
                date: Some(date), ..
            }),
            Some(ResourceLoadable {
                content: Some(Loadable::Ready(meta_items)),
                ..
            }),
        ) => {
            let day_start = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();
            let day_end = date
                .checked_add_days(Days::new(1))
                .and_then(|next_date| next_date.and_hms_opt(0, 0, 0))
                .unwrap_or_default()
                .and_utc();

            meta_items
                .iter()
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
