use crate::types::resource::Subtitles;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, Strict};

// * Deduplication can be achieved by simple comparison (Eq)
// * @TODO Sorting
// * @TODO Serializing/deserializing streams with gzip+base64, for URLs

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl Stream {
    pub fn is_web_ready(&self) -> bool {
        if self.behavior_hints.get("notWebReady") == Some(&serde_json::Value::Bool(true)) {
            return false;
        }
        match &self.source {
            StreamSource::Url { url } if url.starts_with("https:") => true,
            _ => false,
        }
    }
    pub fn is_p2p(&self) -> bool {
        match &self.source {
            StreamSource::Torrent { .. } => true,
            StreamSource::Url { url } if url.starts_with("magnet:") => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: String,
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
        external_url: String,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        player_frame_url: String,
    },
}
