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
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: String,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        info_hash: String,
        file_idx: Option<u16>,
    },
    #[serde(rename_all = "camelCase")]
    External {
        external_url: String,
    },
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
                source: StreamSource::Torrent {
                    info_hash: "07a9de9750158471c3302e4e95edb1107f980fa6".into(),
                    file_idx: Some(1),
                }
            }
        )
    }
}
