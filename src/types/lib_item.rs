use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// @TODO: u64 vs u32
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibItemState {
    pub last_watched: Option<DateTime<Utc>>,
    pub time_watched: u64,
    pub time_offset: u64,
    pub overall_time_watched: u64,
    pub times_watched: u32,
    pub flagged_watched: bool,
    pub duration: u64,
    #[serde(rename = "video_id")]
    pub video_id: Option<String>,
    // @TODO bitfield, special type
    pub watched: Option<String>,
    pub no_notif: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LibItem {
    #[serde(rename = "_id")]
    pub id: String,

    pub removed: bool,
    pub temp: bool,

    #[serde(rename = "_ctime")]
    pub ctime: DateTime<Utc>,
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,

    pub state: LibItemState,

    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub year: Option<String>,
}
