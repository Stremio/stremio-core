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

/// The continue watching section in the app
#[derive(Default, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContinueWatchingPreview {
    pub library_items: Vec<LibraryItem>,
}

impl ContinueWatchingPreview {
    pub fn new(library: &LibraryBucket, notifications: &NotificationsBucket) -> (Self, Effects) {
        let mut library_items = vec![];
        let effects = library_items_update(&mut library_items, library, notifications);
        (Self { library_items }, effects.unchanged())
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for ContinueWatchingPreview {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        // update the CW list if
        match msg {
            // library has changed
            Msg::Internal(Internal::LibraryChanged(true))
            // LibraryItem has been updated (this message alters the `mtime` and will re-order the CW list)
            | Msg::Internal(Internal::UpdateLibraryItem(_))
            // notifications have been updated
            | Msg::Internal(Internal::NotificationsChanged)
            // or a notification has been dismissed (this will re-order the given LibraryItem based on mtime)
            | Msg::Internal(Internal::DismissNotificationItem(_)) => {
                library_items_update(&mut self.library_items, &ctx.library, &ctx.notifications)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn library_items_update(
    library_items: &mut Vec<LibraryItem>,
    library: &LibraryBucket,
    notifications: &NotificationsBucket,
) -> Effects {
    let next_library_items = library
        .items
        .values()
        .filter(|library_item| {
            // either the library item is in CW
            library_item.is_in_continue_watching()
            // or there's a new notification for it
                || notifications
                    .items
                    .get(&library_item.id)
                    .filter(|meta_notifs| !meta_notifs.is_empty())
                    .is_some()
        })
        // either take the oldest video released date or the modification date of the LibraryItem
        .sorted_by(|a, b| {
            let a_time = notifications
                .items
                .get(&a.id)
                .and_then(|notifs| {
                    notifs
                        .values()
                        // take the released date of the video if there is one, or skip this notification
                        .filter_map(|notification| notification.video.released)
                        .sorted_by(|a_released, b_released| {
                            // order by the oldest video released!
                            b_released.cmp(a_released).reverse()
                        })
                        .next()
                })
                .unwrap_or(a.mtime);

            let b_time = notifications
                .items
                .get(&b.id)
                .and_then(|notifs| {
                    notifs
                        .values()
                        // take the released date of the video if there is one, or skip this notification
                        .filter_map(|notification| notification.video.released)
                        .sorted_by(|a_released, b_released| {
                            // order by the oldest video released!
                            b_released.cmp(a_released).reverse()
                        })
                        .next()
                })
                .unwrap_or(b.mtime);

            b_time.cmp(&a_time)
        })
        .take(CATALOG_PREVIEW_SIZE)
        .cloned()
        .collect::<Vec<_>>();

    eq_update(library_items, next_library_items)
}
