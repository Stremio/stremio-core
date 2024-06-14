use std::collections::{hash_map::Entry, HashMap};

use chrono::{DateTime, Duration, Utc};
use futures::FutureExt;
use lazysort::SortedBy;
use once_cell::sync::Lazy;
use serde::Serialize;
use tracing::trace;

use crate::{
    constants::{CALENDAR_IDS_EXTRA_PROP, CALENDAR_ITEMS_COUNT, CALENDAR_STORAGE_KEY},
    models::{
        common::{
            eq_update, resources_update_with_vector_content, Loadable, ResourceLoadable,
            ResourcesAction,
        },
        ctx::{CtxError, CtxStatus},
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx,
    },
    types::{
        addon::{AggrRequest, ExtraType},
        calendar::{
            CalendarBucket, CalendarItem, MAXIMUM_BACKWARD_RELEASE_DATE,
            MAXIMUM_FORWARD_RELEASE_DATE,
        },
        library::LibraryBucket,
        profile::Profile,
        resource::{MetaItem, MetaItemId, VideoId},
    },
};

use super::ctx::Ctx;

static REQUEST_CALENDAR_EVERY: Lazy<Duration> = Lazy::new(|| Duration::hours(6));
#[derive(Serialize, Clone, Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct Calendar {
    /// The calendar Model's bucket
    #[serde(flatten)]
    pub calendar: CalendarBucket,
    #[serde(skip)]
    /// The catalogs response from all addons that support the `calendarVideosIds`
    /// ([`CALENDAR_IDS_EXTRA_PROP`]) resource.
    ///
    /// [`CALENDAR_IDS_EXTRA_PROP`]: static@crate::constants::`CALENDAR_IDS_EXTRA_PROP`
    pub calendar_catalogs: Vec<ResourceLoadable<Vec<MetaItem>>>,
}

impl Calendar {
    pub fn new(calendar: CalendarBucket) -> Self {
        Self {
            calendar,
            calendar_catalogs: vec![],
        }
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for Calendar {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        update_calendar::<E>(
            &mut self.calendar,
            &mut self.calendar_catalogs,
            &ctx.profile,
            &ctx.library,
            &ctx.status,
            msg,
        )
    }
}

pub fn update_calendar<E: Env + 'static>(
    calendar: &mut CalendarBucket,
    calendar_catalogs: &mut Vec<ResourceLoadable<Vec<MetaItem>>>,
    profile: &Profile,
    library: &LibraryBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Load(ActionLoad::Calendar)) => {
            Effects::msg(Msg::Internal(Internal::PullCalendar)).unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::PullCalendar)) => {
            Effects::msg(Msg::Internal(Internal::PullCalendar)).unchanged()
        }
        Msg::Internal(Internal::PullCalendar) => {
            let (reason, should_make_request) = match calendar.last_updated {
                Some(last_updated) if last_updated + *REQUEST_CALENDAR_EVERY <= E::now() => (
                    format!(
                        "`true` since {last_updated} + {hours} hours <= {now}",
                        hours = REQUEST_CALENDAR_EVERY.num_hours(),
                        now = E::now()
                    ),
                    true,
                ),
                None => ("`true` since last updated is `None`".to_string(), true),
                Some(last_updated) => (
                    format!(
                        "`false` since {last_updated} + {hours} hours > {now}",
                        hours = REQUEST_CALENDAR_EVERY.num_hours(),
                        now = E::now()
                    ),
                    false,
                ),
            };

            tracing::debug!(
                name = "Calendar",
                reason = reason,
                last_updated = calendar.last_updated.as_ref().map(ToString::to_string),
                hours = REQUEST_CALENDAR_EVERY.num_hours(),
                "Should calendar addon resource be called? {should_make_request}"
            );

            let sorted_library_items_id_types = library
                .items
                .values()
                .filter(|library_item| library_item.should_pull_notifications())
                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                .map(|library_item| (library_item.id.to_owned(), library_item.r#type.to_owned()))
                .collect::<Vec<_>>();

            let calendar_catalog_resource_effects =
                if !sorted_library_items_id_types.is_empty() && should_make_request {
                    trace!(
                        "Sorted by `mtime` LibraryItem id and type: {:?}",
                        sorted_library_items_id_types
                    );
                    let catalog_resource_effects = resources_update_with_vector_content::<E, _>(
                        calendar_catalogs,
                        // force the making of a requests every time PullNotifications is called.
                        ResourcesAction::force_request(
                            &AggrRequest::CatalogsFiltered(vec![ExtraType::Ids {
                                extra_name: CALENDAR_IDS_EXTRA_PROP.name.to_owned(),
                                id_types: sorted_library_items_id_types,
                                limit: Some(CALENDAR_ITEMS_COUNT),
                            }]),
                            &profile.addons,
                        ),
                    );

                    calendar.last_updated = Some(E::now());

                    catalog_resource_effects
                } else {
                    Effects::none().unchanged()
                };

            // first update the calendar items
            let calendar_items_effects =
                update_calendar_items::<E>(calendar, calendar_catalogs, library);

            // because calendar are getting loaded by forcing new requests
            // we do not trigger a `CalendarChanged` as the addons should return results first.
            calendar_catalog_resource_effects
                .join(calendar_items_effects)
                .unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
            let calendar_catalogs_effects = eq_update(calendar_catalogs, vec![]);
            let next_calendar = CalendarBucket::new::<E>(profile.uid(), vec![]);
            let calendar_effects = if *calendar != next_calendar {
                *calendar = next_calendar;
                Effects::msg(Msg::Internal(Internal::CalendarChanged))
            } else {
                Effects::none().unchanged()
            };
            calendar_catalogs_effects.join(calendar_effects).unchanged()
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(_))
                if loading_auth_request == auth_request =>
            {
                let calendar_catalogs_effects = eq_update(calendar_catalogs, vec![]);
                let next_calendar = CalendarBucket::new::<E>(profile.uid(), vec![]);
                let calendar_effects = if *calendar != next_calendar {
                    *calendar = next_calendar;
                    Effects::msg(Msg::Internal(Internal::CalendarChanged))
                } else {
                    Effects::none().unchanged()
                };

                let pull_calendar_effects =
                    Effects::msg(Msg::Internal(Internal::PullCalendar)).unchanged();
                calendar_catalogs_effects
                    .join(calendar_effects)
                    .join(pull_calendar_effects)
                    .unchanged()
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
            let calendar_catalogs_effects = resources_update_with_vector_content::<E, _>(
                calendar_catalogs,
                ResourcesAction::ResourceRequestResult { request, result },
            );
            let calendar_items_effects = if calendar_catalogs_effects.has_changed {
                update_calendar_items::<E>(calendar, calendar_catalogs, library)
            } else {
                Effects::none().unchanged()
            };

            let calendar_effects = if calendar_items_effects.has_changed {
                Effects::msg(Msg::Internal(Internal::CalendarChanged))
            } else {
                Effects::none().unchanged()
            };
            calendar_catalogs_effects
                .join(calendar_items_effects)
                .join(calendar_effects)
        }
        Msg::Internal(Internal::CalendarChanged) => {
            Effects::one(push_calendar_to_storage::<E>(calendar)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn update_calendar_items<E: Env + 'static>(
    // calendar_items: &mut HashMap<MetaItemId, HashMap<VideoId, CalendarItem>>,
    calendar: &mut CalendarBucket,
    calendar_catalogs: &[ResourceLoadable<Vec<MetaItem>>],
    library: &LibraryBucket,
) -> Effects {
    let next_calendar_items = calendar.update_calendar_items::<E>(calendar_catalogs, library);

    eq_update(&mut calendar.items, next_calendar_items)
}

impl CalendarBucket {
    fn update_calendar_items<E: Env + 'static>(
        &self,
        calendar_catalogs: &[ResourceLoadable<Vec<MetaItem>>],
        library: &LibraryBucket,
    ) -> HashMap<MetaItemId, HashMap<VideoId, CalendarItem>> {
        let selected_catalogs = calendar_catalogs
            .iter()
            // take all catalogs with successful result or error
            .filter(|catalog| {
                matches!(
                    &catalog.content,
                    Some(Loadable::Ready(_)) | Some(Loadable::Err(_))
                )
            })
            .collect::<Vec<_>>();

        let next_calendar_items =
            library
                .items
                .iter()
                .fold(HashMap::new(), |mut map, (meta_id, library_item)| {
                    // Exit early if we don't need to pull calendar for the library item
                    if !library_item.should_pull_notifications() {
                        return map;
                    }

                    // find the first occurrence of the meta item inside the catalogs
                    let meta_item = match selected_catalogs.iter().find_map(|catalog| {
                        catalog
                            .content
                            .as_ref()
                            .and_then(|content| content.ready())
                            .and_then(|content| {
                                content.iter().find(|meta_item| {
                                    &meta_item.preview.id == meta_id && !meta_item.videos.is_empty()
                                })
                            })
                    }) {
                        Some(meta_item) => meta_item,
                        _ => {
                            // try to default to currently existing calendar in the bucket before returning
                            match self.items.get(meta_id) {
                                Some(existing_calendar) if !existing_calendar.is_empty() => {
                                    let filtered_current_calendar: HashMap<String, CalendarItem> =
                                        existing_calendar
                                            .iter()
                                            .filter_map(|(video_id, calendar_item)| {
                                                // filter by the same requirements as new videos
                                                // to remove videos that no longer match
                                                if should_retain_video_released::<E>(Some(
                                                    &calendar_item.video_released,
                                                ))
                                                .is_some()
                                                {
                                                    Some((
                                                        video_id.to_owned(),
                                                        calendar_item.to_owned(),
                                                    ))
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                    map.insert(meta_id.to_owned(), filtered_current_calendar);
                                }
                                _ => {
                                    // in any other case - skip it, e.g. meta_id not found or empty calendar
                                }
                            }

                            return map;
                        }
                    };

                    let mut meta_calendar = map.entry(meta_id.to_owned()).or_default();

                    // meta items videos
                    meta_item
                        .videos_iter()
                        // We need to manually `fold()` instead of `collect()`,
                        // otherwise the last seen element with a given key
                        // will be present in the final HashMap instead of the first occurrence.
                        .fold(&mut meta_calendar, |meta_calendar, video| {
                            // filter by the same requirements as new videos
                            // to remove videos that no longer match

                            match (video.released, video.series_info.as_ref()) {
                                // make sure video_released is in 2 months forwards or backwards
                                (Some(video_released), Some(series_info))
                                    if should_retain_video_released::<E>(Some(&video_released))
                                        .is_some() =>
                                {
                                    let calendar_entry = meta_calendar.entry(video.id.to_owned());

                                    // for now just skip same videos that already exist
                                    // leave the first one found in the Vec.
                                    if let Entry::Vacant(new) = calendar_entry {
                                        let calendar = CalendarItem {
                                            meta_id: meta_id.to_owned(),
                                            meta_name: meta_item.preview.name.to_owned(),
                                            video_title: video.title.to_owned(),
                                            video_id: video.id.to_owned(),
                                            series_info: series_info.to_owned(),
                                            video_released,
                                        };

                                        new.insert(calendar);
                                    }
                                }
                                _ => {}
                            }

                            meta_calendar
                        });

                    // if not videos were added and the hashmap is empty, just remove the MetaItem record all together
                    if meta_calendar.is_empty() {
                        map.remove(meta_id);
                    }

                    map
                });

        next_calendar_items
    }
}

fn push_calendar_to_storage<E: Env + 'static>(calendar: &CalendarBucket) -> Effect {
    let ids = calendar.items.keys().cloned().collect();
    EffectFuture::Sequential(
        E::set_storage(CALENDAR_STORAGE_KEY, Some(calendar))
            .map(move |result| match result {
                Ok(_) => Msg::Event(Event::CalendarPushedToStorage { ids }),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::CalendarPushedToStorage { ids }),
                }),
            })
            .boxed_env(),
    )
    .into()
}

/// Shared function to decide if a given video should be included in calendar
/// or excluded.
///
/// - [`MAXIMUM_FORWARD_RELEASE_DATE`]
/// - [`MAXIMUM_BACKWARD_RELEASE_DATE`]
///
/// # Returns
///
/// The video_released DateTime extracted from the arguments if it should be retained
fn should_retain_video_released<E: Env>(
    video_released: Option<&DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    match video_released {
        Some(video_released)
            if *video_released > E::now() - MAXIMUM_FORWARD_RELEASE_DATE
                && *video_released < E::now() + MAXIMUM_BACKWARD_RELEASE_DATE =>
        {
            Some(*video_released)
        }
        _ => None,
    }
}
