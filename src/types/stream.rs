use serde_derive::*;
use serde_hex::{SerHex, Strict};

// * Deduplication can be achieved by simple comparison (Eq)
// * @TODO Sorting
// * @TODO Serializing/deserializing streams with gzip+base64, for URLs

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    #[serde(flatten)]
    pub source: StreamSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtitles: Vec<SubtitlesSource>,
    // @TODO better data structure
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

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
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
    // This can also be used to point to internal pages,
    // using stremio:// URLs
    #[serde(rename_all = "camelCase")]
    External {
        external_url: String,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        player_frame_url: String,
    },
}

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct SubtitlesSource {
    pub id: String,
    // @TODO: ISO 639-2
    pub lang: String,
    pub url: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn deserialize() {
        let stream_json = "{\"infoHash\":\"07a9de9750158471c3302e4e95edb1107f980fa6\",\"fileIdx\":1,\"title\":\"test stream\"}";
        let stream: Stream = serde_json::from_str(&stream_json).unwrap();
        assert_eq!(
            stream,
            Stream {
                title: Some("test stream".into()),
                thumbnail: None,
                subtitles: Default::default(),
                behavior_hints: Default::default(),
                source: StreamSource::Torrent {
                    info_hash: [
                        0x07, 0xa9, 0xde, 0x97, 0x50, 0x15, 0x84, 0x71, 0xc3, 0x30, 0x2e, 0x4e,
                        0x95, 0xed, 0xb1, 0x10, 0x7f, 0x98, 0x0f, 0xa6
                    ],
                    file_idx: Some(1),
                }
            }
        )
    }

    #[test]
    pub fn deserialize_no_fileidx() {
        let stream_json =
            "{\"infoHash\":\"07a9de9750158471c3302e4e95edb1107f980fa6\",\"title\":\"test stream\"}";
        let stream: Stream = serde_json::from_str(&stream_json).unwrap();
        assert_eq!(
            stream,
            Stream {
                title: Some("test stream".into()),
                thumbnail: None,
                subtitles: Default::default(),
                behavior_hints: Default::default(),
                source: StreamSource::Torrent {
                    info_hash: [
                        0x07, 0xa9, 0xde, 0x97, 0x50, 0x15, 0x84, 0x71, 0xc3, 0x30, 0x2e, 0x4e,
                        0x95, 0xed, 0xb1, 0x10, 0x7f, 0x98, 0x0f, 0xa6
                    ],
                    file_idx: None,
                }
            }
        )
    }
}
