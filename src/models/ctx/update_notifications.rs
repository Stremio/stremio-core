use std::collections::{hash_map::Entry, HashMap};

use chrono::{DateTime, Duration, Utc};
use futures::FutureExt;
use lazysort::SortedBy;
use once_cell::sync::Lazy;
use tracing::trace;

use crate::{
    constants::{LAST_VIDEOS_IDS_EXTRA_PROP, NOTIFICATIONS_STORAGE_KEY, NOTIFICATION_ITEMS_COUNT},
    models::{
        common::{
            eq_update, resources_update_with_vector_content, Loadable, ResourceLoadable,
            ResourcesAction,
        },
        ctx::{CtxError, CtxStatus},
    },
    runtime::{
        msg::{Action, ActionCtx, Event, Internal, Msg},
        Effect, EffectFuture, Effects, Env, EnvFutureExt,
    },
    types::{
        addon::{AggrRequest, ExtraType},
        library::LibraryBucket,
        notifications::{NotificationItem, NotificationsBucket},
        profile::Profile,
        resource::{MetaItem, MetaItemId, VideoId},
    },
};

static REQUEST_LAST_VIDEOS_EVERY: Lazy<Duration> = Lazy::new(|| Duration::hours(6));

pub fn update_notifications<E: Env + 'static>(
    notifications: &mut NotificationsBucket,
    notification_catalogs: &mut Vec<ResourceLoadable<Vec<MetaItem>>>,
    profile: &Profile,
    library: &LibraryBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::PullNotifications)) => {
            Effects::msg(Msg::Internal(Internal::PullNotifications)).unchanged()
        }
        Msg::Internal(Internal::PullNotifications) => {
            let (reason, should_make_request) = match notifications.last_updated {
                Some(last_updated) if last_updated + *REQUEST_LAST_VIDEOS_EVERY <= E::now() => (
                    format!(
                        "`true` since {last_updated} + {hours} hours <= {now}",
                        hours = REQUEST_LAST_VIDEOS_EVERY.num_hours(),
                        now = E::now()
                    ),
                    true,
                ),
                None => ("`true` since last updated is `None`".to_string(), true),
                Some(last_updated) => (
                    format!(
                        "`false` since {last_updated} + {hours} hours > {now}",
                        hours = REQUEST_LAST_VIDEOS_EVERY.num_hours(),
                        now = E::now()
                    ),
                    false,
                ),
            };

            tracing::debug!(
                name = "Notifications",
                reason = reason,
                last_updated = notifications.last_updated.as_ref().map(ToString::to_string),
                hours = REQUEST_LAST_VIDEOS_EVERY.num_hours(),
                "Should last-videos addon resource be called? {should_make_request}"
            );

            let sorted_library_items_id_types = library
                .items
                .values()
                .filter(|library_item| library_item.should_pull_notifications())
                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                .map(|library_item| (library_item.id.to_owned(), library_item.r#type.to_owned()))
                .collect::<Vec<_>>();

            let notifications_catalog_resource_effects =
                if !sorted_library_items_id_types.is_empty() && should_make_request {
                    trace!(
                        "Sorted by `mtime` LibraryItem id and type: {:?}",
                        sorted_library_items_id_types
                    );
                    let catalog_resource_effects = resources_update_with_vector_content::<E, _>(
                        notification_catalogs,
                        // force the making of a requests every time PullNotifications is called.
                        ResourcesAction::force_request(
                            &AggrRequest::CatalogsFiltered(vec![ExtraType::Ids {
                                extra_name: LAST_VIDEOS_IDS_EXTRA_PROP.name.to_owned(),
                                id_types: sorted_library_items_id_types,
                                limit: Some(NOTIFICATION_ITEMS_COUNT),
                            }]),
                            &profile.addons,
                        ),
                    );

                    notifications.last_updated = Some(E::now());

                    catalog_resource_effects
                } else {
                    Effects::none().unchanged()
                };

            // first update the notification items
            let notification_items_effects = update_notification_items::<E>(
                &mut notifications.items,
                notification_catalogs,
                library,
            );

            // because notifications are getting loaded by forcing new requests
            // we do not trigger a `NotificationsChanged` as the addons should return results first.
            notifications_catalog_resource_effects
                .join(notification_items_effects)
                .unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::DismissNotificationItem(id))) => Effects::msg(
            Msg::Internal(Internal::DismissNotificationItem(id.to_owned())),
        )
        .unchanged(),
        Msg::Internal(Internal::Logout(_)) => {
            let notification_catalogs_effects = eq_update(notification_catalogs, vec![]);
            let next_notifications = NotificationsBucket::new::<E>(profile.uid(), vec![]);
            let notifications_effects = if *notifications != next_notifications {
                *notifications = next_notifications;
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
            } else {
                Effects::none().unchanged()
            };
            notification_catalogs_effects
                .join(notifications_effects)
                .unchanged()
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(_))
                if loading_auth_request == auth_request =>
            {
                let notification_catalogs_effects = eq_update(notification_catalogs, vec![]);
                let next_notifications = NotificationsBucket::new::<E>(profile.uid(), vec![]);
                let notifications_effects = if *notifications != next_notifications {
                    *notifications = next_notifications;
                    Effects::msg(Msg::Internal(Internal::NotificationsChanged))
                } else {
                    Effects::none().unchanged()
                };

                let pull_notifications_effects =
                    Effects::msg(Msg::Internal(Internal::PullNotifications)).unchanged();
                notification_catalogs_effects
                    .join(notifications_effects)
                    .join(pull_notifications_effects)
                    .unchanged()
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
            let notification_catalogs_effects = resources_update_with_vector_content::<E, _>(
                notification_catalogs,
                ResourcesAction::ResourceRequestResult { request, result },
            );
            let notification_items_effects = if notification_catalogs_effects.has_changed {
                update_notification_items::<E>(
                    &mut notifications.items,
                    notification_catalogs,
                    library,
                )
            } else {
                Effects::none().unchanged()
            };

            let notifications_effects = if notification_items_effects.has_changed {
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
            } else {
                Effects::none().unchanged()
            };
            notification_catalogs_effects
                .join(notification_items_effects)
                .join(notifications_effects)
        }
        Msg::Internal(Internal::DismissNotificationItem(id)) => {
            dismiss_notification_item::<E>(library, notifications, id)
        }
        Msg::Internal(Internal::NotificationsChanged) => {
            Effects::one(push_notifications_to_storage::<E>(notifications)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn update_notification_items<E: Env + 'static>(
    notification_items: &mut HashMap<MetaItemId, HashMap<VideoId, NotificationItem>>,
    notification_catalogs: &[ResourceLoadable<Vec<MetaItem>>],
    library: &LibraryBucket,
) -> Effects {
    let selected_catalogs = notification_catalogs
        .iter()
        // take all catalogs with successful result or error
        .filter(|catalog| {
            matches!(
                &catalog.content,
                Some(Loadable::Ready(_)) | Some(Loadable::Err(_))
            )
        })
        .collect::<Vec<_>>();

    // shared function to decide if a given video should be included in notifications
    // or excluded
    // returns the video_released DateTime extracted from the arguments if it should be retained
    let should_retail_video_released = |last_watched: Option<&DateTime<Utc>>,
                                        video_released: Option<&DateTime<Utc>>|
     -> Option<DateTime<Utc>> {
        match (last_watched, video_released) {
            (Some(last_watched), Some(video_released)) => {
                if last_watched < video_released &&
                        // exclude future videos (i.e. that will air in the future)
                        video_released <= &E::now()
                {
                    Some(*video_released)
                } else {
                    None
                }
            }
            // if you've never watched an episode, then we want to include new videos
            (None, Some(video_released)) => Some(*video_released),
            _ => None,
        }
    };

    let next_notification_items =
        library
            .items
            .iter()
            .fold(HashMap::new(), |mut map, (meta_id, library_item)| {
                // Exit early if we don't need to pull notifications for the library item
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
                        // try to default to currently existing notifications in the bucket before returning
                        match notification_items.get(meta_id) {
                            Some(existing_notifications) if !existing_notifications.is_empty() => {
                                let filtered_current_notifs = existing_notifications
                                    .iter()
                                    .filter_map(|(video_id, notif_item)| {
                                        // filter by the same requirements as new videos
                                        // to remove videos that no longer match
                                        if should_retail_video_released(
                                            library_item.state.last_watched.as_ref(),
                                            Some(&notif_item.video_released),
                                        )
                                        .is_some()
                                        {
                                            Some((video_id.to_owned(), notif_item.to_owned()))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                map.insert(meta_id.to_owned(), filtered_current_notifs);
                            }
                            _ => {
                                // in any other case - skip it, e.g. meta_id not found or empty notifications
                            }
                        }

                        return map;
                    }
                };

                let mut meta_notifs: &mut HashMap<_, _> =
                    map.entry(meta_id.to_owned()).or_default();

                // meta items videos
                meta_item
                    .videos_iter()
                    .filter_map(
                        |video| match (&library_item.state.last_watched, video.released) {
                            (Some(last_watched), Some(video_released)) => {
                                if should_retail_video_released(
                                    Some(last_watched),
                                    Some(&video_released),
                                )
                                .is_some()
                                {
                                    Some((&library_item.id, &video.id, video_released))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                    )
                    // We need to manually `fold()` instead of `collect()`,
                    // otherwise the last seen element with a given key
                    // will be present in the final HashMap instead of the first occurrence.
                    .fold(
                        &mut meta_notifs,
                        |meta_notifs, (meta_id, video_id, video_released)| {
                            let notif_entry = meta_notifs.entry(video_id.to_owned());

                            // for now just skip same videos that already exist
                            // leave the first one found in the Vec.
                            if let Entry::Vacant(new) = notif_entry {
                                let notification = NotificationItem {
                                    meta_id: meta_id.to_owned(),
                                    video_id: video_id.to_owned(),
                                    video_released,
                                };

                                new.insert(notification);
                            }

                            meta_notifs
                        },
                    );

                // if not videos were added and the hashmap is empty, just remove the MetaItem record all together
                if meta_notifs.is_empty() {
                    map.remove(meta_id);
                }

                map
            });

    eq_update(notification_items, next_notification_items)
}

fn push_notifications_to_storage<E: Env + 'static>(notifications: &NotificationsBucket) -> Effect {
    let ids = notifications.items.keys().cloned().collect();
    EffectFuture::Sequential(
        E::set_storage(NOTIFICATIONS_STORAGE_KEY, Some(notifications))
            .map(move |result| match result {
                Ok(_) => Msg::Event(Event::NotificationsPushedToStorage { ids }),
                Err(error) => Msg::Event(Event::Error {
                    error: CtxError::from(error),
                    source: Box::new(Event::NotificationsPushedToStorage { ids }),
                }),
            })
            .boxed_env(),
    )
    .into()
}

fn dismiss_notification_item<E: Env + 'static>(
    library: &LibraryBucket,
    notifications: &mut NotificationsBucket,
    id: &str,
) -> Effects {
    match notifications.items.remove(id) {
        Some(_) => {
            // when dismissing notifications, make sure we update the `last_watched`
            // of the LibraryItem this way if we've only `DismissedNotificationItem`
            // the next time we `PullNotifications` we won't see the same notifications.
            let library_item_effects = match library.items.get(id) {
                Some(library_item) => {
                    let mut library_item = library_item.to_owned();
                    library_item.state.last_watched = Some(E::now());

                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(library_item)))
                        .unchanged()
                }
                _ => Effects::none().unchanged(),
            };

            Effects::msg(Msg::Internal(Internal::NotificationsChanged))
                .join(library_item_effects)
                .join(Effects::msg(Msg::Event(Event::NotificationsDismissed {
                    id: id.to_owned(),
                })))
        }
        _ => Effects::none().unchanged(),
    }
}
