use super::stream::*;
use chrono::{DateTime, Utc};
use serde_derive::*;

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetaPreview {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub name: String,
    pub poster: Option<String>,
    // @TODO maybe this should be an enum?
    pub poster_shape: Option<String>,
}

// https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/meta.md#meta-object
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
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
    #[serde(default)]
    pub popularity: f64,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub poster_shape: Option<String>,
    // @TODO: default to one video
    #[serde(default)]
    pub videos: Vec<Video>,
    // This is a video id; the case of the video not being in .videos must be handled at runtime
    pub featured_vid: Option<String>,
    // @TODO: decide between HashMap or a Vec(String, String)
    // @TODO: consider using a URL type
    // NOTE: this is also used instead of "website"
    #[serde(default)]
    pub external_urls: Vec<(String, String)>,
    // @TODO use some ISO language type
    //pub language: Option<String>,
    // @TODO crew
    // @TODO genres
}

// https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/meta.md#video-object
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(alias = "name")]
    pub title: String,
    pub released: DateTime<Utc>,
    pub overview: Option<String>,
    pub thumbnail: Option<String>,
    pub streams: Option<Vec<Stream>>,
    // @TODO: season AND episode (but they have to go together)
    #[serde(flatten)]
    pub series_info: Option<SeriesInfo>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}
