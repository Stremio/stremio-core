use serde_derive::*;
use chrono::{DateTime, Utc};
use super::stream::*;

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

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, Default)]
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
    pub poster_shape: Option<String>,
    // @TODO: default to one video
    #[serde(default)]
    pub videos: Vec<Video>,
    // @TODO: other
    // @TODO videos
    // @TODO crew
}


#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub name: String,
    pub released: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub poster: Option<String>,
    pub streams: Option<Vec<Stream>>,
}
