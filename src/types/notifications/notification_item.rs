use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::resource::{MetaItemId, VideoId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    /// Notification meta item id
    pub meta_id: MetaItemId,
    pub video_id: VideoId,
    pub video_released: DateTime<Utc>,
}
