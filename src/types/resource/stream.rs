use crate::types::resource::Subtitles;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, Strict};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    #[serde(flatten)]
    pub source: StreamSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtitles: Vec<Subtitles>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub behavior_hints: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: Url,
    },
    #[serde(rename_all = "camelCase")]
    YouTube {
        yt_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        #[serde(with = "SerHex::<Strict>")]
        info_hash: [u8; 20],
        file_idx: Option<u16>,
    },
    #[serde(rename_all = "camelCase")]
    External {
        external_url: Url,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        player_frame_url: Url,
    },
}
