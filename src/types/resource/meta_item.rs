use crate::types::resource::{PosterShape, Stream, Video};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
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
    // @TODO: default to one video
    #[serde(default)]
    pub videos: Vec<Video>,
    // This is a video id; the case of the video not being in .videos must be handled at runtime
    pub featured_vid: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
    // @TODO use some ISO language type
    // pub language: Option<String>,
    #[serde(default)]
    pub trailers: Vec<Stream>,
    #[serde(default)]
    pub behavior_hints: MetaBehaviorHints,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemPreview {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub name: String,
    pub poster: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub released: Option<DateTime<Utc>>,
    #[serde(default)]
    pub poster_shape: PosterShape,
    #[serde(default)]
    pub trailers: Vec<Stream>,
    #[serde(default)]
    pub behavior_hints: MetaBehaviorHints,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    name: String,
    category: String,
    url: Url,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaBehaviorHints {
    pub default_video_id: Option<String>,
}
