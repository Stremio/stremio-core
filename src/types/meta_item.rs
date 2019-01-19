use serde_derive::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaPreview {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    name: String,
    poster: Option<String>,
}

// @TODO: should we derive Hash, Eq?
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    name: String,
    poster: Option<String>,
    background: Option<String>,
    logo: Option<String>,
    #[serde(default)]
    popularity: f64,
    description: Option<String>,
    release_info: Option<String>,
    // @TODO: other
    // @TODO videos
    // @TODO crew
}
