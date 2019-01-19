use serde_derive::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaPreview {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub name: String,
    pub poster: Option<String>,
}

// @TODO: should we derive Hash, Eq?
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    #[serde(default)]
    pub popularity: f64,
    pub description: Option<String>,
    pub release_info: Option<String>,
    // @TODO: other
    // @TODO videos
    // @TODO crew
}
