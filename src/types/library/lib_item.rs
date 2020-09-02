use crate::types::empty_string_as_none;
use crate::types::resource::{MetaBehaviorHints, PosterShape};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibItemState {
    #[serde(deserialize_with = "empty_string_as_none")]
    pub last_watched: Option<DateTime<Utc>>,
    pub time_watched: u64,
    pub time_offset: u64,
    pub overall_time_watched: u64,
    pub times_watched: u32,
    // @TODO: consider bool that can be deserialized from an integer
    pub flagged_watched: u32,
    pub duration: u64,
    #[serde(rename = "video_id", deserialize_with = "empty_string_as_none")]
    pub video_id: Option<String>,
    // @TODO bitfield, special type
    #[serde(deserialize_with = "empty_string_as_none")]
    pub watched: Option<String>,
    // release date of last observed video
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub last_vid_released: Option<DateTime<Utc>>,
    pub no_notif: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibItem {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub poster: Option<String>,
    #[serde(default)]
    pub poster_shape: PosterShape,
    pub removed: bool,
    pub temp: bool,
    #[serde(rename = "_ctime", deserialize_with = "empty_string_as_none", default)]
    pub ctime: Option<DateTime<Utc>>,
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
    pub state: LibItemState,
    #[serde(default)]
    pub behavior_hints: MetaBehaviorHints,
}

impl LibItem {
    pub fn should_sync(&self) -> bool {
        !self.removed || self.state.overall_time_watched > 60_000
    }
    pub fn is_in_continue_watching(&self) -> bool {
        self.should_sync() && (!self.removed || self.temp) && self.state.time_offset > 0
    }
}
