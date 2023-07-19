use serde::{Deserialize, Serialize};

use crate::types::{
    resource::MetaItemId,
    resource::{Video, VideoId},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationItem {
    /// Notification meta item id
    pub meta_id: MetaItemId,
    pub video_id: VideoId,
    pub video: Video,
}
