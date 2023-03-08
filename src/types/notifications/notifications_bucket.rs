use crate::runtime::Env;
use crate::types::notifications::NotificationItem;
use crate::types::profile::UID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct NotificationsBucket {
    pub uid: UID,
    /// Notifications videos
    ///
    /// The list key is the `MetaItem.preview.id`.
    ///
    /// Videos in this list are already released (released < Now)
    /// and newer than `LibraryItem.state.last_video_released`.
    pub items: HashMap<String, NotificationItem>,
    pub created: DateTime<Utc>,
}

impl NotificationsBucket {
    pub fn new<E: Env + 'static>(uid: UID, items: Vec<NotificationItem>) -> Self {
        NotificationsBucket {
            uid,
            items: items
                .into_iter()
                .map(|item| (item.id.to_owned(), item))
                .collect(),
            created: E::now(),
        }
    }
}
