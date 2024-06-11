use std::collections::{hash_map::Entry, HashMap};

#[cfg(test)]
use chrono::offset::TimeZone;
use chrono::{DateTime, Utc};
#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::{
    runtime::Env,
    types::{
        profile::UID,
        resource::{MetaItemId, VideoId},
    },
};

use super::CalendarItem;

/// - Calendar items for previous (up to ~1 month) and future (up to ~2 months) episodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(rename_all = "camelCase")]
pub struct CalendarBucket {
    pub uid: UID,
    /// Calendar per MetaItem
    pub items: HashMap<MetaItemId, HashMap<VideoId, CalendarItem>>,
    /// The last time notifications were pulled.
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
    /// The moment that the notification bucket was initialized.
    #[cfg_attr(test, derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()")))]
    pub created: DateTime<Utc>,
}

impl CalendarBucket {
    // todo: add calendar init. argument
    pub fn new<E: Env + 'static>(uid: UID, items: Vec<CalendarItem>) -> Self {
        Self {
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
            last_updated: None,
            created: E::now(),
        }
    }
}
