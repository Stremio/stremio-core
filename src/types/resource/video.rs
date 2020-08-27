use crate::types::resource::Stream;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(alias = "name")]
    pub title: String,
    pub released: DateTime<Utc>,
    pub overview: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub streams: Vec<Stream>,
    #[serde(flatten)]
    pub series_info: Option<SeriesInfo>,
    #[serde(default)]
    pub trailers: Vec<Stream>,
}
