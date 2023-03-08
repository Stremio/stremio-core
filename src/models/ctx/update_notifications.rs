use std::collections::HashMap;

use crate::{
    constants::{LAST_VIDEOS_IDS_EXTRA_PROP, NOTIFICATIONS_STORAGE_KEY},
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
        addon::{AggrRequest, ExtraValue},
        library::LibraryBucket,
        notifications::{NotificationItem, NotificationsBucket},
        profile::Profile,
        resource::MetaItem,
    },
};

use either::Either;
use enclose::enclose;
use futures::FutureExt;
use lazysort::SortedBy;

pub fn update_notifications<E: Env + 'static>(
    notifications: &mut NotificationsBucket,
    last_videos_catalogs: &mut Vec<ResourceLoadable<Vec<MetaItem>>>,
    profile: &Profile,
    library: &LibraryBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::PullNotifications)) => {
            let notification_item_ids = library
                .items
                .values()
                .filter(|library_item| library_item.should_pull_notifications())
                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                // Why would we take only X amount based on Options limit and do nothing with the rest?!
                // .take(*LAST_VIDEOS_IDS_EXTRA_PROP.options_limit)
                .map(|library_item| &library_item.id)
                .cloned()
                .collect::<Vec<_>>();
            let extra = notification_item_ids
                .iter()
                .map(|id| ExtraValue {
                    name: LAST_VIDEOS_IDS_EXTRA_PROP.name.to_owned(),
                    value: id.to_owned(),
                })
                .collect::<Vec<_>>();
            let last_videos_catalogs_effects = resources_update_with_vector_content::<E, _>(
                last_videos_catalogs,
                ResourcesAction::ResourcesRequested {
                    request: &AggrRequest::AllCatalogs {
                        extra: &extra,
                        r#type: &None,
                    },
                    addons: &profile.addons,
                },
            );
            notifications.created = E::now();
            notifications.items = notification_item_ids
                .iter()
                .filter_map(|id| notifications.items.get_key_value(id))
                .map(|(id, notification_item)| (id.to_owned(), notification_item.to_owned()))
                .collect();
            let notification_items_effects = update_notification_items::<E>(
                &mut notifications.items,
                &last_videos_catalogs,
                &library,
            );
            let notifications_effects = if notification_items_effects.has_changed {
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
                    .unchanged()
                    .join(notification_items_effects)
            } else {
                notification_items_effects
            };
            last_videos_catalogs_effects.join(notifications_effects)
        }
        Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
            let next_notifications = NotificationsBucket::new::<E>(profile.uid(), vec![]);
            let notifications_effects = if *notifications != next_notifications {
                *notifications = next_notifications;
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
            } else {
                Effects::none().unchanged()
            };
            let last_videos_catalogs_effects = eq_update(last_videos_catalogs, vec![]);
            notifications_effects.join(last_videos_catalogs_effects)
        }
        Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => match (status, result) {
            (CtxStatus::Loading(loading_auth_request), Ok(_))
                if loading_auth_request == auth_request =>
            {
                let next_notifications = NotificationsBucket::new::<E>(profile.uid(), vec![]);
                let notifications_effects = if *notifications != next_notifications {
                    *notifications = next_notifications;
                    Effects::msg(Msg::Internal(Internal::NotificationsChanged))
                } else {
                    Effects::none().unchanged()
                };
                let last_videos_catalogs_effects = eq_update(last_videos_catalogs, vec![]);
                notifications_effects.join(last_videos_catalogs_effects)
            }
            _ => Effects::none().unchanged(),
        },
        Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
            let last_videos_catalogs_effects = resources_update_with_vector_content::<E, _>(
                last_videos_catalogs,
                ResourcesAction::ResourceRequestResult { request, result },
            );
            let notification_items_effects = update_notification_items::<E>(
                &mut notifications.items,
                &last_videos_catalogs,
                &library,
            );
            let notifications_effects = if notification_items_effects.has_changed {
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
                    .unchanged()
                    .join(notification_items_effects)
            } else {
                notification_items_effects
            };
            last_videos_catalogs_effects.join(notifications_effects)
        }
        Msg::Internal(Internal::NotificationsChanged) => {
            Effects::one(push_notifications_to_storage::<E>(notifications)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn update_notification_items<E: Env + 'static>(
    notification_items: &mut HashMap<String, NotificationItem>,
    last_videos_catalogs: &Vec<ResourceLoadable<Vec<MetaItem>>>,
    library: &LibraryBucket,
) -> Effects {
    let selected_catalogs = last_videos_catalogs
        .iter()
        .take_while(|catalog| match &catalog.content {
            Some(Loadable::Ready(_)) | Some(Loadable::Err(_)) => true,
            _ => false,
        })
        .collect::<Vec<_>>();

    let next_notification_items = last_videos_catalogs
        // get first addon results - Cinemeta
        .first()
        // get resources where the extra is for Last videos ids
        .map(|resource| &resource.request.path.extra)
        .map(|extra| Either::Left(extra.iter()))
        .unwrap_or_else(|| Either::Right(std::iter::empty()))
        .filter(|extra_value| extra_value.name == LAST_VIDEOS_IDS_EXTRA_PROP.name)
        // extra value is the MetaItem id
        .map(|extra_value| &extra_value.value)
        .filter_map(|id| {
            if let Some(notification_item) = notification_items.get(id) {
                return Some(notification_item.to_owned());
            };

            let library_item = library.items.get(id);
            // find the MetaItem in the loaded catalogs
            let meta_item = selected_catalogs.iter().find_map(|catalog| {
                catalog
                    .content
                    .as_ref()
                    .and_then(|content| content.ready())
                    .and_then(|content| {
                        content.iter().find(|meta_item| meta_item.preview.id == *id)
                    })
            });
            match (library_item, meta_item) {
                (Some(library_item), Some(meta_item)) if !meta_item.videos.is_empty() => {
                    let mut videos = if meta_item.is_series() {
                        Either::Left(meta_item.videos.iter())
                    } else {
                        Either::Right(meta_item.videos.iter().rev())
                    }
                    .into_iter();

                    let video = match library_item.state.last_video_released.as_ref() {
                        Some(last_video_released) => videos.find(|video| {
                            video
                                .released
                                .as_ref()
                                // find videos that are already released and newer than last_video_released of the LibraryItem
                                .map(|released| {
                                    released > last_video_released && released <= &E::now()
                                })
                                .unwrap_or_default()
                        }),
                        // or just return the first video from the iterator
                        None => videos.next(),
                    };

                    video.map(|video| NotificationItem {
                        id: id.to_owned(),
                        video: video.to_owned(),
                    })
                }
                _ => None,
            }
        })
        .map(|notification_item| (notification_item.id.to_owned(), notification_item))
        .collect();
    eq_update(notification_items, next_notification_items)
}

fn push_notifications_to_storage<E: Env + 'static>(notifications: &NotificationsBucket) -> Effect {
    EffectFuture::Sequential(
        E::set_storage(NOTIFICATIONS_STORAGE_KEY, Some(notifications))
            .map(
                enclose!((notifications.uid => uid) move |result| match result {
                    Ok(_) => Msg::Event(Event::ProfilePushedToStorage { uid }),
                    Err(error) => Msg::Event(Event::Error {
                        error: CtxError::from(error),
                        source: Box::new(Event::ProfilePushedToStorage { uid }),
                    })
                }),
            )
            .boxed_env(),
    )
    .into()
}
