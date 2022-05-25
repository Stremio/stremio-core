use crate::types::deserialize_single_as_vec;
use crate::types::resource::Stream;
use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

fn deserialize_and_sort_videos<'de, D>(deserializer: D) -> Result<Vec<Video>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut videos: Vec<Video> = Vec::<Video>::deserialize(deserializer)?;
    let some_sessons = videos.iter().any(|v| v.series_info.is_some());

    videos.sort_by(|a, b| {
        if let (Some(a), Some(b)) = (a.series_info.as_ref(), b.series_info.as_ref()) {
            let a_season = if a.season == 0 { u32::MAX } else { a.season };
            let b_season = if b.season == 0 { u32::MAX } else { b.season };
            a_season.cmp(&b_season).then(a.episode.cmp(&b.episode))
        } else if a.series_info.is_some() {
            std::cmp::Ordering::Less
        } else if b.series_info.is_some() {
            std::cmp::Ordering::Greater
        } else if let (Some(a), Some(b)) = (a.released, b.released) {
            if some_sessons {
                a.cmp(&b)
            } else {
                b.cmp(&a)
            }
        } else if a.released.is_some() {
            std::cmp::Ordering::Less
        } else if b.released.is_some() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    Ok(videos)
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    #[serde(flatten)]
    pub preview: MetaItemPreview,
    #[serde(default, deserialize_with = "deserialize_and_sort_videos")]
    pub videos: Vec<Video>,
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
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
