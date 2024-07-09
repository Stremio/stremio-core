use lazysort::SortedBy;
use serde::Serialize;

use crate::{
    constants::CATALOG_PREVIEW_SIZE,
    models::{common::eq_update, ctx::Ctx},
    runtime::{
        msg::{Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        library::{LibraryBucket, LibraryItem},
        notifications::NotificationsBucket,
    },
};

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(flatten)]
    pub library_item: LibraryItem,
    /// a count of the total notifications we have for this item
    pub notifications: usize,
}

/// The continue watching section in the app
#[derive(Default, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContinueWatchingPreview {
    pub items: Vec<Item>,
}

impl ContinueWatchingPreview {
    pub fn new(library: &LibraryBucket, notifications: &NotificationsBucket) -> (Self, Effects) {
        let mut items = vec![];
        let effects = library_items_update(&mut items, library, notifications);
        (Self { items }, effects.unchanged())
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for ContinueWatchingPreview {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        // update the CW list if
        match msg {
            // library has changed
            Msg::Internal(Internal::LibraryChanged(true))
            // notifications have been updated
            | Msg::Internal(Internal::NotificationsChanged) => {
                library_items_update(&mut self.items, &ctx.library, &ctx.notifications)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn library_items_update(
    cw_items: &mut Vec<Item>,
    library: &LibraryBucket,
    notifications: &NotificationsBucket,
) -> Effects {
    let next_cw_items = library
        .items
        .values()
        .filter_map(|library_item| {
            let library_notification = notifications
                .items
                .get(&library_item.id)
                .filter(|meta_notifs| !meta_notifs.is_empty());

            // either the library item is in CW
            if library_item.is_in_continue_watching()
            // or there's a new notification for it
                || library_notification.is_some()
            {
                Some((
                    library_item,
                    library_notification
                        .map(|notifs| notifs.len())
                        .unwrap_or_default(),
                ))
            } else {
                None
            }
        })
        // either take the oldest video released date or the modification date of the LibraryItem
        .sorted_by(|(item_a, _), (item_b, _)| {
            let a_time = notifications
                .items
                .get(&item_a.id)
                .and_then(|notifs| {
                    notifs
                        .values()
                        // take the video released date of the notification
                        .map(|notification| notification.video_released)
                        // order by the newest video released!
                        .sorted_by(|a_released, b_released| b_released.cmp(a_released))
                        .next()
                })
                .unwrap_or(item_a.mtime);

            let b_time = notifications
                .items
                .get(&item_b.id)
                .and_then(|notifs| {
                    notifs
                        .values()
                        // take the video released date of the notification
                        .map(|notification| notification.video_released)
                        // order by the newest video released!
                        .sorted_by(|a_released, b_released| b_released.cmp(a_released))
                        .next()
                })
                .unwrap_or(item_b.mtime);

            b_time.cmp(&a_time)
        })
        .take(CATALOG_PREVIEW_SIZE)
        .map(|(library_item, notifications)| Item {
            library_item: library_item.clone(),
            notifications,
        })
        .collect::<Vec<_>>();

    eq_update(cw_items, next_cw_items)
}
