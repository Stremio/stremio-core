use crate::types::resource::Video;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationItem {
    /// Notification video id
    pub id: String,
    pub video: Video,
}
