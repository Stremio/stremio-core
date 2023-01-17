use crate::constants::YOUTUBE_ADDON_ID_PREFIX;
use crate::types::resource::Subtitles;
use boolinator::Boolinator;
#[cfg(test)]
use derivative::Derivative;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use magnet_url::Magnet;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DefaultOnNull};
use std::collections::HashMap;
use std::io::Write;
use stremio_serde_hex::{SerHex, Strict};
use url::Url;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    #[serde(flatten)]
    pub source: StreamSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(alias = "title", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtitles: Vec<Subtitles>,
    #[serde(default, skip_serializing_if = "is_default_value")]
    pub behavior_hints: StreamBehaviorHints,
}

impl Stream {
    pub fn magnet_url(&self) -> Option<Magnet> {
        match &self.source {
            StreamSource::Torrent {
                info_hash,
                announce,
                ..
            } => Some(Magnet {
                dn: self.name.to_owned(),
                hash_type: Some("btih".to_string()),
                xt: Some(hex::encode(info_hash)),
                xl: None,
                tr: announce
                    .iter()
                    .filter(|source| source.starts_with("tracker:"))
                    .map(|tracker| tracker.replace("tracker:", ""))
                    .collect::<Vec<String>>(),
                kt: None,
                ws: None,
                acceptable_source: None,
                mt: None,
                xs: None,
            }),
            _ => None,
        }
    }
    pub fn encode(&self) -> Result<String, anyhow::Error> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
        let stream = serde_json::to_string(&self)?;
        encoder.write_all(stream.as_bytes())?;
        let stream = encoder.finish()?;
        let stream = base64::encode(stream);
        Ok(stream)
    }
    pub fn decode(stream: String) -> Result<Self, anyhow::Error> {
        let stream = base64::decode(stream)?;
        let mut writer = Vec::new();
        let mut decoder = ZlibDecoder::new(writer);
        decoder.write_all(&stream)?;
        writer = decoder.finish()?;
        let stream = String::from_utf8(writer)?;
        let stream = serde_json::from_str(&stream)?;
        Ok(stream)
    }
    pub fn youtube(video_id: &str) -> Option<Self> {
        video_id
            .starts_with(YOUTUBE_ADDON_ID_PREFIX)
            .as_option()
            // video id is in format: yt_id:YT_CHANNEL_ID:YT_VIDEO_ID
            .and_then(|_| video_id.split(':').nth(2))
            .map(|yt_id| Self {
                source: StreamSource::YouTube {
                    yt_id: yt_id.to_owned(),
                },
                name: None,
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: Default::default(),
            })
    }
    pub fn m3u_data_uri(&self, url: String) -> String {
        format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            base64::encode(format!("#EXTM3U\n#EXTINF:0\n{}", url))
        )
    }
    pub fn to_streaming_url(&self, streaming_server_url: &Url) -> Option<String> {
        match &self.source {
            StreamSource::Torrent {
                info_hash,
                file_idx,
                announce,
            } => Some(
                streaming_server_url
                    .join(&format!("{}/", hex::encode(info_hash)))
                    .and_then(|url| match file_idx {
                        Some(idx) => url.join(&format!("{}/", idx)),
                        None => Ok(url),
                    })
                    .and_then(|url| url.join(&format!("?tr={}", announce.join("&tr="))))
                    .map(|url| url.to_string())
                    .expect("Failed to build streaming url for torrent"),
            ),
            StreamSource::YouTube { yt_id } => {
                Some(format!("{}yt/{}", streaming_server_url, yt_id))
            }
            _ => None,
        }
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(untagged)]
pub enum StreamSource {
    Url {
        url: Url,
    },
    #[cfg_attr(test, derivative(Default))]
    #[serde(rename_all = "camelCase")]
    YouTube {
        yt_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        #[serde(with = "SerHex::<Strict>")]
        info_hash: [u8; 20],
        file_idx: Option<u16>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, alias = "sources")]
        announce: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        player_frame_url: Url,
    },
    #[serde(
        rename_all = "camelCase",
        deserialize_with = "deserialize_stream_source_external"
    )]
    External {
        #[serde(skip_serializing_if = "Option::is_none")]
        external_url: Option<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        android_tv_url: Option<Url>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tizen_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        webos_url: Option<String>,
    },
}

type ExternalStreamSource = (Option<Url>, Option<Url>, Option<String>, Option<String>);

fn deserialize_stream_source_external<'de, D>(
    deserializer: D,
) -> Result<ExternalStreamSource, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Helper {
        external_url: Option<Url>,
        android_tv_url: Option<Url>,
        tizen_url: Option<String>,
        webos_url: Option<String>,
    }
    let source = Helper::deserialize(deserializer)?;
    if source.external_url.is_none()
        && source.android_tv_url.is_none()
        && source.tizen_url.is_none()
        && source.webos_url.is_none()
    {
        return Err(D::Error::custom("Invalid StreamSource::External"));
    };
    Ok((
        source.external_url,
        source.android_tv_url,
        source.tizen_url,
        source.webos_url,
    ))
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamProxyHeaders {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub request: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub response: HashMap<String, String>,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamBehaviorHints {
    #[serde(default, skip_serializing_if = "is_default_value")]
    pub not_web_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binge_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_whitelist: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_headers: Option<StreamProxyHeaders>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

fn is_default_value<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
