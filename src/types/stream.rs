use serde_derive::*;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    // @TODO: all props from the new spec
    pub name: Option<String>,
    #[serde(flatten)]
    pub source: StreamSource,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum StreamSource {
    Url{ url: String },
    Torrent{ info_hash: String, file_idx: Option<u16> },
    External{ external_url: String },
}
