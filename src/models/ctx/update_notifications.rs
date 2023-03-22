use crate::constants::{
    LAST_VIDEOS_IDS_EXTRA_PROP, NOTIFICATIONS_STORAGE_KEY, NOTIFICATION_ITEMS_COUNT,
};
use crate::models::common::{
    eq_update, resources_update_with_vector_content, Loadable, ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::{CtxError, CtxStatus};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt};
use crate::types::addon::{AggrRequest, ExtraValue};
use crate::types::library::LibraryBucket;
use crate::types::notifications::{NotificationItem, NotificationsBucket};
use crate::types::profile::Profile;
use crate::types::resource::MetaItem;
use either::Either;
use futures::FutureExt;
use lazysort::SortedBy;
use std::collections::HashMap;

pub fn update_notifications<E: Env + 'static>(
    notifications: &mut NotificationsBucket,
    notification_catalogs: &mut Vec<ResourceLoadable<Vec<MetaItem>>>,
    profile: &Profile,
    library: &LibraryBucket,
    status: &CtxStatus,
    msg: &Msg,
) -> Effects {
    match msg {
        Msg::Action(Action::Ctx(ActionCtx::PullNotificatons)) => {
            let library_item_ids = library
                .items
                .values()
                .filter(|library_item| library_item.should_pull_notifications())
                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                .take(NOTIFICATION_ITEMS_COUNT)
                .map(|library_item| &library_item.id)
                .cloned()
                .collect::<Vec<_>>();
            let notification_catalogs_effects = resources_update_with_vector_content::<E, _>(
                notification_catalogs,
                ResourcesAction::ResourcesRequested {
                    request: &AggrRequest::AllCatalogs {
                        extra: &vec![ExtraValue {
                            name: LAST_VIDEOS_IDS_EXTRA_PROP.name.to_owned(),
                            value: library_item_ids.join(","),
                        }],
                        r#type: &None,
                    },
                    addons: &profile.addons,
                },
            );
            notifications.created = E::now();
            let notification_items_effects = update_notification_items(
                &mut notifications.items,
                &notification_catalogs,
                &library,
            );
            let notifications_effects = if notification_items_effects.has_changed {
                Effects::msg(Msg::Internal(Internal::NotificationsChanged))
            } else {
                Effects::none().unchanged()
            };
            notification_catalogs_effects
                .join(notification_items_effects)
                .join(notifications_effects)
                .unchanged()
        }
        Msg::Action(Action::Ctx(ActionCtx::Logout)) | Msg::Internal(Internal::Logout) => {
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
                notification_catalogs_effects
                    .join(notifications_effects)
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
                update_notification_items(
                    &mut notifications.items,
                    &notification_catalogs,
                    &library,
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
                .unchanged()
        }
        Msg::Internal(Internal::DismissNotificationItem(id)) => {
            match notifications.items.remove(id) {
                Some(_) => Effects::msg(Msg::Internal(Internal::NotificationsChanged)).unchanged(),
                _ => Effects::none().unchanged(),
            }
        }
        Msg::Internal(Internal::NotificationsChanged) => {
            Effects::one(push_notifications_to_storage::<E>(notifications)).unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

fn update_notification_items(
    notification_items: &mut HashMap<String, NotificationItem>,
    notification_catalogs: &Vec<ResourceLoadable<Vec<MetaItem>>>,
    library: &LibraryBucket,
) -> Effects {
    let selected_catalogs = notification_catalogs
        .iter()
        .take_while(|catalog| match &catalog.content {
            Some(Loadable::Ready(_)) | Some(Loadable::Err(_)) => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    let next_notification_ids = notification_catalogs
        .first()
        .map(|resource| &resource.request.path.extra)
        .map(|extra| Either::Left(extra.iter()))
        .unwrap_or_else(|| Either::Right(std::iter::empty()))
        .find(|extra_value| extra_value.name == LAST_VIDEOS_IDS_EXTRA_PROP.name)
        .map(|extra_value| Either::Left(extra_value.value.split(",")))
        .unwrap_or_else(|| Either::Right(std::iter::empty()));
    let next_notification_items = next_notification_ids
        .filter_map(|id| {
            if let Some(notification_item) = notification_items.get(id) {
                return Some(notification_item.to_owned());
            };

            let library_item = library.items.get(id);
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
                (Some(library_item), Some(meta_item))
                    if library_item.should_pull_notifications() && !meta_item.videos.is_empty() =>
                {
                    meta_item
                        .videos_iter()
                        .find(|video| {
                            match (&library_item.state.last_video_released, &video.released) {
                                (Some(last_video_released), Some(video_released)) => {
                                    video_released > last_video_released
                                }
                                _ => false,
                            }
                        })
                        .map(|video| NotificationItem {
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
