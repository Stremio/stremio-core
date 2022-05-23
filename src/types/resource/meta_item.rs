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
pub struct MetaItem {
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
    #[serde(default, deserialize_with = "deserialize_and_sort_videos")]
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
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod meta_item_videos_tests {
    use super::*;

    #[test]
    fn videos_minimal() {
        let json = r#"{
        "id": "", "type": "series", "name": "",
        "videos": [
            {"id":"2", "title":""},
            {"id":"1", "title":""},
            {"id":"3", "title":""}
        ]
    }"#;
        let item: MetaItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.videos.len(), 3);
        // No data to sort agains. The ordering is from the addon
        assert_eq!(item.videos[0].id, "2".to_string());
        assert_eq!(item.videos[1].id, "1".to_string());
        assert_eq!(item.videos[2].id, "3".to_string());
    }

    #[test]
    fn videos_released_equal() {
        let json = r#"{
        "id": "", "type": "series", "name": "",
        "videos": [
            {"id":"2", "title":"", "released": "2022-01-01T00:00:00+00:00"},
            {"id":"1", "title":"", "released": "2022-01-01T00:00:00+00:00"},
            {"id":"3", "title":"", "released": "2022-01-01T00:00:00+00:00"}
        ]
    }"#;
        let item: MetaItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.videos.len(), 3);
        // All have same date. The ordering is from the addon
        assert_eq!(item.videos[0].id, "2".to_string());
        assert_eq!(item.videos[1].id, "1".to_string());
        assert_eq!(item.videos[2].id, "3".to_string());
    }

    #[test]
    fn videos_released_sequal() {
        let json = r#"{
        "id": "", "type": "series", "name": "",
        "videos": [
            {"id":"nd1", "title":""},
            {"id":"nd2", "title":""},
            {"id":"2", "title":"", "released": "2022-02-01T00:00:00+00:00"},
            {"id":"1", "title":"", "released": "2022-01-01T00:00:00+00:00"},
            {"id":"3", "title":"", "released": "2022-03-01T00:00:00+00:00"}
        ]
    }"#;
        let item: MetaItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.videos.len(), 5);
        // There is no series_info. Order by date descending.
        // If no date - at the end and the order is ddefined by addon
        assert_eq!(item.videos[0].id, "3".to_string());
        assert_eq!(item.videos[1].id, "2".to_string());
        assert_eq!(item.videos[2].id, "1".to_string());
        assert_eq!(item.videos[3].id, "nd1".to_string());
        assert_eq!(item.videos[4].id, "nd2".to_string());
    }

    #[test]
    fn various_videos_deserialization() {
        let json = r#"{
        "id": "", "type": "series", "name": "",
        "videos": [
            {"id":"special2", "title":"", "released": "2022-05-01T00:00:00+00:00", "season": 0, "episode": 2},
            {"id":"S01E02", "title":"", "released": "2022-02-01T00:00:00+00:00", "season": 1, "episode": 2},
            {"id":"special1", "title":"", "released": "2022-05-01T00:00:00+00:00", "season": 0, "episode": 1},
            {"id":"S01E01", "title":"", "released": "2022-01-01T00:00:00+00:00", "season": 1, "episode": 1},
            {"id":"M2", "title":"", "released": "2022-02-01T00:00:00+00:00"},
            {"id":"M1", "title":"", "released": "2022-01-01T00:00:00+00:00"},
            {"id":"nd1", "title":""},
            {"id":"nd2", "title":""},
            {"id":"S02E01", "title":"", "released": "2022-03-01T00:00:00+00:00", "season": 2, "episode": 1}
        ]
    }"#;
        let item: MetaItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.videos.len(), 9);
        // Sort by season, then episode. Special at the end.
        // If no series_info sort by date ascending
        // If no date - sort to the end. Preserver order from addon
        assert_eq!(item.videos[0].id, "S01E01".to_string());
        assert_eq!(item.videos[1].id, "S01E02".to_string());
        assert_eq!(item.videos[2].id, "S02E01".to_string());
        assert_eq!(item.videos[3].id, "special1".to_string());
        assert_eq!(item.videos[4].id, "special2".to_string());
        assert_eq!(item.videos[5].id, "M1".to_string());
        assert_eq!(item.videos[6].id, "M2".to_string());
        assert_eq!(item.videos[7].id, "nd1".to_string());
        assert_eq!(item.videos[8].id, "nd2".to_string());
    }
}
