use std::collections::{hash_map::Entry, HashMap, HashSet};

#[cfg(test)]
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::{
    runtime::Env,
    types::{
        notifications::NotificationItem,
        profile::UID,
        resource::{MetaItemId, VideoId, Video},
    },
};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CalendarItem {
    pub meta_item: MetaItemId,
    pub video_id: VideoId,
    pub video_released: DateTime<Utc>,
}

// pub const CALENDAR_BACKWARDS_LIMIT = Duration::

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
/// Notification bucket using the `lastVideos` resource of user's addons
///
/// This bucket will extract from the addon responses:
/// - Notifications for new episodes (movie series)
/// - Calendar items for previous (up to 1 month) and future (up to 2 months) episodes
pub struct NotificationsBucket {
    #[serde(default)]
    pub uid: UID,
    /// Notifications per meta item and video id
    #[serde(default)]
    pub items: HashMap<MetaItemId, HashMap<VideoId, NotificationItem>>,
    /// Calendar per MetaItem
    pub calendar: HashMap<MetaItemId, HashMap<VideoId, CalendarItem>>,
    /// The last time notifications were pulled.
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
    /// The moment that the notification bucket was initialized.
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()")))]
    pub created: DateTime<Utc>,
}

impl NotificationsBucket {
    // todo: add calendar init. argument
    pub fn new<E: Env + 'static>(uid: UID, items: Vec<NotificationItem>) -> Self {
        NotificationsBucket {
            uid,
            items: items.into_iter().fold(HashMap::new(), |mut acc, item| {
                let meta_notifs: &mut HashMap<_, _> = acc.entry(item.meta_id.clone()).or_default();

                let notif_entry = meta_notifs.entry(item.video_id.clone());

                // for now just skip same videos that already exist
                // leave the first one found in the Vec.
                if let Entry::Vacant(new) = notif_entry {
                    new.insert(item);
                }

                acc
            }),
            calendar: Default::default(),
            last_updated: None,
            created: E::now(),
        }
    }
}
