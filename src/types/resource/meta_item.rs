use crate::types::deserialize_single_as_vec;
use crate::types::resource::Stream;
use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    pub id: String,
    pub r#type: String,
    #[serde(default)]
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub popularity: Option<f64>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub released: Option<DateTime<Utc>>,
    #[serde(default)]
    pub poster_shape: PosterShape,
    #[serde(default)]
    pub videos: Vec<Video>,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub trailer_streams: Vec<Stream>,
    #[serde(default)]
    pub behavior_hints: MetaItemBehaviorHints,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct MetaItemPreview {
    pub id: String,
    pub r#type: String,
    #[serde(default)]
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub released: Option<DateTime<Utc>>,
    #[serde(default)]
    pub poster_shape: PosterShape,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub trailer_streams: Vec<Stream>,
    #[serde(default)]
    pub behavior_hints: MetaItemBehaviorHints,
}

#[derive(Derivative, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Square,
    Landscape,
    #[derivative(Default)]
    #[serde(other)]
    Poster,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(default, alias = "name")]
    pub title: String,
    pub released: Option<DateTime<Utc>>,
    pub overview: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(
        alias = "stream",
        deserialize_with = "deserialize_single_as_vec",
        default
    )]
    pub streams: Vec<Stream>,
    #[serde(flatten)]
    pub series_info: Option<SeriesInfo>,
    #[serde(default)]
    pub trailer_streams: Vec<Stream>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Link {
    pub name: String,
    pub category: String,
    pub url: Url,
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct MetaItemBehaviorHints {
    #[serde(default)]
    pub default_video_id: Option<String>,
    #[serde(default)]
    pub featured_video_id: Option<String>,
    #[serde(default)]
    pub has_scheduled_videos: bool,
}
