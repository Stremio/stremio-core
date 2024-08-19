use chrono::{DateTime, Utc};
#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::types::resource::{MetaItemId, SeriesInfo, VideoId};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[serde(rename_all = "camelCase")]
pub struct CalendarItem {
    pub meta_id: MetaItemId,
    pub meta_name: String,
    pub video_id: VideoId,
    pub video_title: String,
    pub series_info: SeriesInfo,
    /// The date at which this episode gets released (future)
    /// or has been released (past).
    #[cfg_attr(test, derivative(Default(value = "Utc::now()")))]
    pub video_released: DateTime<Utc>,
}
