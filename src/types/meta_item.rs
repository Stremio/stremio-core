use super::stream::*;
use chrono::{DateTime, Utc};
use serde_derive::*;

// Type can be represented as:
//#[derive(Deserialize, Serialize, Debug)]
//#[serde(rename_all = "camelCase")]
//pub enum Preset { Movie, Series, Channel, Tv }
//#[derive(Deserialize, Serialize, Debug)]
//#[serde(untagged)]
//pub enum ItemType {
//    Preset(Preset),
//    Other(String)
//}
// or, some day it may be done with 1 enum:
// https://users.rust-lang.org/t/catchall-variant-in-serde/20748
// https://github.com/serde-rs/serde/pull/1382

#[derive(Default, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorHints {
    default_video_id: Option<String>,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Poster,
    Square,
    Landscape,
    #[serde(other)]
    Unspecified,
}

impl Default for PosterShape {
    fn default() -> Self {
        PosterShape::Unspecified
    }
}

impl PosterShape {
    pub fn is_unspecified(&self) -> bool {
        *self == PosterShape::Unspecified
    }
    // @TODO: auto-derive this?
    pub fn to_str(&self) -> &'static str {
        match self {
            PosterShape::Poster => "poster",
            PosterShape::Square => "square",
            PosterShape::Landscape => "landscape",
            PosterShape::Unspecified => "poster",
        }
    }
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    name: String,
    category: String,
    url: String,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
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
    pub trailer: Option<Stream>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetaPreview {
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
    #[serde(default, skip_serializing_if = "PosterShape::is_unspecified")]
    pub poster_shape: PosterShape,
    pub trailer: Option<Stream>,
    #[serde(default)]
    pub behavior_hints: BehaviorHints,
}

#[derive(PartialEq, Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaDetail {
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
    #[serde(default, skip_serializing_if = "PosterShape::is_unspecified")]
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
    pub trailer: Option<Stream>,
    #[serde(default)]
    pub behavior_hints: BehaviorHints,
}
