use std::collections::HashMap;

use chrono::{DateTime, Days, Duration, NaiveDate, Utc};
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
    /// Guide date in the user's local timezone;
    /// when `None`, defaults to the local today
    pub date: Option<NaiveDate>,
    /// The user's timezone offset in minutes east of UTC
    /// (e.g. `120` for UTC+2); the local day is resolved to a UTC
    /// window and shows are bucketed into it
    #[serde(default)]
    pub utc_offset: i32,
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
    /// Today in the user's local timezone
    pub today: Option<NaiveDate>,
    /// The next channels page request (with the `skip` extra; the `date`
    /// extra is appended per overlapping UTC date when loaded);
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
                        utc_offset,
                    }) => catalog_update::<E>(
                        &mut self.catalog,
                        CatalogPageRequest::First,
                        request,
                        date,
                        *utc_offset,
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
                match (self.selected.as_ref(), self.selectable.next_page.as_ref()) {
                    (
                        Some(Selected {
                            date: Some(date),
                            utc_offset,
                            ..
                        }),
                        Some(next_page),
                    ) => {
                        let catalog_effects = catalog_update::<E>(
                            &mut self.catalog,
                            CatalogPageRequest::Next,
                            &next_page.request,
                            date,
                            *utc_offset,
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
    let utc_offset = next_selected
        .as_ref()
        .map(|selected| selected.utc_offset)
        .unwrap_or_default();
    let date = next_selected
        .as_ref()
        .and_then(|selected| selected.date)
        .or_else(|| Some(local_date_now::<E>(utc_offset)));

    eq_update(
        selected,
        Some(Selected {
            request,
            date,
            utc_offset,
        }),
    )
}

fn catalog_update<E: Env + 'static>(
    catalog: &mut Catalog,
    page_request: CatalogPageRequest,
    request: &ResourceRequest,
    date: &NaiveDate,
    utc_offset: i32,
) -> Effects {
    let mut effects = Effects::none().unchanged();
    let mut pages = vec![];
    // the local day resolves to a UTC window which may span two UTC
    // dates - a page is fetched per overlapping UTC date and the shows
    // are bucketed back into the local day by `channels_update`
    for utc_date in overlapping_utc_dates(date, utc_offset) {
        let request = with_date_extra(request, &utc_date);
        let mut page = ResourceLoadable {
            request: request.to_owned(),
            content: None,
        };
        effects = effects.join(resource_update_with_vector_content::<E, MetaItem>(
            &mut page,
            ResourceAction::ResourceRequested { request: &request },
        ));
        pages.push(page);
    }
    match page_request {
        CatalogPageRequest::First => *catalog = pages,
        CatalogPageRequest::Next => catalog.extend(pages),
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
    let utc_offset = selected
        .as_ref()
        .map(|selected| selected.utc_offset)
        .unwrap_or_default();
    let next_page = next_page_update(selected, catalog, profile);
    let updated_selectable = Selectable {
        catalogs,
        prev_date: date.and_then(|date| date.checked_sub_days(Days::new(1))),
        next_date: date.and_then(|date| date.checked_add_days(Days::new(1))),
        today: Some(local_date_now::<E>(utc_offset)),
        next_page,
    };

    eq_update(selectable, updated_selectable)
}

fn next_page_update(
    selected: &Option<Selected>,
    catalog: &Catalog,
    profile: &Profile,
) -> Option<SelectablePage> {
    let request = match selected {
        Some(Selected {
            request: Some(request),
            date: Some(_),
            ..
        }) => request,
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
                })
                .collect::<Option<Vec<_>>>()
        })
        // pages of the overlapping UTC dates carry the same channels -
        // the skip offset counts them once
        .map(|pages| {
            pages
                .into_iter()
                .flatten()
                .unique_by(|meta_item| meta_item.preview.id.to_owned())
                .count()
        })
        .map(|skip| SelectablePage {
            request: ResourceRequest {
                base: request.base.to_owned(),
                path: ResourcePath {
                    extra: request
                        .path
                        .extra
                        .to_owned()
                        .extend_one(&SKIP_EXTRA_PROP, Some(skip.to_string())),
                    ..request.path.to_owned()
                },
            },
        })
}

fn channels_update(
    channels: &mut Vec<ChannelGuide>,
    selected: &Option<Selected>,
    catalog: &Catalog,
) -> Effects {
    let updated_channels = match selected {
        Some(Selected {
            date: Some(date),
            utc_offset,
            ..
        }) => {
            let (day_start, day_end) = local_day_window(date, *utc_offset);

            // pages of the overlapping UTC dates carry the same channels
            // with different programs - merge their shows per channel,
            // keeping the channel order of first appearance
            let mut updated_channels = Vec::<ChannelGuide>::new();
            let mut channel_indexes = HashMap::<String, usize>::new();
            let meta_items = catalog
                .iter()
                .filter_map(|page| page.content.as_ref().and_then(|content| content.ready()))
                .flatten();
            for meta_item in meta_items {
                // include only program shows overlapping the local day window
                let shows = meta_item.videos.iter().filter(|video| {
                    video.epg_info.as_ref().is_some_and(|epg_info| {
                        epg_info.start_time < day_end && epg_info.end_time > day_start
                    })
                });
                match channel_indexes.get(&meta_item.preview.id) {
                    Some(channel_index) => updated_channels[*channel_index]
                        .shows
                        .extend(shows.cloned()),
                    None => {
                        channel_indexes
                            .insert(meta_item.preview.id.to_owned(), updated_channels.len());
                        updated_channels.push(ChannelGuide {
                            channel: meta_item.preview.to_owned(),
                            shows: shows.cloned().collect(),
                        });
                    }
                };
            }
            for channel_guide in updated_channels.iter_mut() {
                channel_guide.shows = channel_guide
                    .shows
                    .drain(..)
                    // a show spanning UTC midnight is returned for both dates
                    .unique_by(|video| video.id.to_owned())
                    .sorted_by_key(|video| {
                        video.epg_info.as_ref().map(|epg_info| epg_info.start_time)
                    })
                    .collect();
            }

            updated_channels
        }
        _ => vec![],
    };

    eq_update(channels, updated_channels)
}

/// Guide catalogs of the installed `epgProvider` addons as
/// `(addon_name, catalog_name, request)`, requests without the `date` extra;
/// only catalogs declaring the `date` extra are guide catalogs - epgProvider
/// addons may expose regular catalogs alongside their guides
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
                .filter(|manifest_catalog| manifest_catalog.is_epg_guide())
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

/// Today's date in the user's local timezone
fn local_date_now<E: Env>(utc_offset: i32) -> NaiveDate {
    (E::now() + Duration::minutes(utc_offset as i64)).date_naive()
}

/// The UTC window `[start, end)` of the user's local `date`
fn local_day_window(date: &NaiveDate, utc_offset: i32) -> (DateTime<Utc>, DateTime<Utc>) {
    let day_start = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc()
        - Duration::minutes(utc_offset as i64);

    (day_start, day_start + Duration::days(1))
}

/// The UTC dates the user's local `date` overlaps -
/// two dates when local midnight is not aligned with UTC midnight
fn overlapping_utc_dates(date: &NaiveDate, utc_offset: i32) -> Vec<NaiveDate> {
    let (day_start, day_end) = local_day_window(date, utc_offset);

    [
        day_start.date_naive(),
        (day_end - Duration::seconds(1)).date_naive(),
    ]
    .into_iter()
    .unique()
    .collect()
}

/// The guide request for the given date
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
