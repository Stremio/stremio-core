use crate::types::resource::Video;
use serde::{Deserialize, Serialize};

pub type MetaItemId = String;
pub type VideoId = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationItem {
    /// Notification meta item id
    pub meta_id: MetaItemId,
    pub video_id: VideoId,
    pub video: Video,
}
