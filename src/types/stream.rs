use serde_derive::*;
use serde_hex::{SerHex, Strict};

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub subtitles: Vec<SubtitlesSource>,
    #[serde(flatten)]
    pub source: StreamSource,
    // @TODO behavior_hints
}

#[derive(Eq, PartialEq, Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: String,
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
        player_frame_url: String
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
        let stream_json = "{\"infoHash\":\"07a9de9750158471c3302e4e95edb1107f980fa6\",\"fileIdx\":1,\"name\":\"test stream\"}";
        let stream: Stream = serde_json::from_str(&stream_json).unwrap();
        assert_eq!(
            stream,
            Stream {
                name: Some("test stream".into()),
                description: None,
                thumbnail: None,
                subtitles: vec![],
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
}
