use crate::constants::{BASE64, URI_COMPONENT_ENCODE_SET, YOUTUBE_ADDON_ID_PREFIX};
use crate::types::resource::Subtitles;
use base64::Engine;
use boolinator::Boolinator;
#[cfg(test)]
use derivative::Derivative;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use magnet_url::Magnet;
use percent_encoding::utf8_percent_encode;
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
            StreamSource::Url { url } if url.scheme() == "magnet" => Magnet::new(url.as_str()).ok(),
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
                    // `tracker` and `dht` prefixes are used internally by the server.js
                    // we need to remove those prefixes when generating the magnet URL
                    .map(|tracker| {
                        tracker
                            .strip_prefix("tracker:")
                            .map(ToString::to_string)
                            .unwrap_or_else(|| tracker.to_owned())
                    })
                    .map(|tracker| {
                        tracker
                            .strip_prefix("dht:")
                            .map(ToString::to_string)
                            .unwrap_or_else(|| tracker.to_owned())
                    })
                    .map(|tracker| {
                        utf8_percent_encode(&tracker, URI_COMPONENT_ENCODE_SET).to_string()
                    })
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
        let stream = BASE64.encode(stream);
        Ok(stream)
    }
    pub fn decode(stream: String) -> Result<Self, anyhow::Error> {
        let stream = BASE64.decode(stream)?;
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
    pub fn download_url(&self) -> Option<String> {
        match &self.source {
            StreamSource::Url { url } if url.scheme() == "magnet" => {
                self.magnet_url().map(|magnet_url| magnet_url.to_string())
            }
            StreamSource::Url { url } => Some(url.to_string()),
            StreamSource::Torrent { .. } => {
                self.magnet_url().map(|magnet_url| magnet_url.to_string())
            }
            StreamSource::YouTube { .. } => self.youtube_url(),
            StreamSource::External { external_url, .. } => {
                external_url.as_ref().map(|url| url.to_string())
            }
            StreamSource::PlayerFrame { player_frame_url } => Some(player_frame_url.to_string()),
        }
    }
    pub fn m3u_data_uri(&self, streaming_server_url: Option<&Url>) -> Option<String> {
        self.streaming_url(streaming_server_url).map(|url| {
            format!(
                "data:application/octet-stream;charset=utf-8;base64,{}",
                BASE64.encode(format!("#EXTM3U\n#EXTINF:0\n{url}"))
            )
        })
    }
    pub fn streaming_url(&self, streaming_server_url: Option<&Url>) -> Option<String> {
        match (&self.source, streaming_server_url) {
            (StreamSource::Url { url }, _) if url.scheme() != "magnet" => Some(url.to_string()),
            (
                StreamSource::Torrent {
                    info_hash,
                    file_idx,
                    announce,
                },
                Some(streaming_server_url),
            ) => {
                let mut url = streaming_server_url.to_owned();
                match url.path_segments_mut() {
                    Ok(mut path) => {
                        path.push(&hex::encode(info_hash));
                        if let Some(file_idx) = file_idx {
                            path.push(&file_idx.to_string());
                        }
                    }
                    _ => return None,
                };
                if !announce.is_empty() {
                    let mut query = url.query_pairs_mut();
                    query.extend_pairs(announce.iter().map(|tracker| ("tr", tracker)));
                };
                Some(url.to_string())
            }
            (StreamSource::YouTube { yt_id }, Some(streaming_server_url)) => {
                let mut url = streaming_server_url.to_owned();
                match url.path_segments_mut() {
                    Ok(mut path) => {
                        path.push("yt");
                        path.push(
                            &utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET).to_string(),
                        );
                    }
                    _ => return None,
                };
                Some(url.to_string())
            }
            _ => None,
        }
    }
    pub fn youtube_url(&self) -> Option<String> {
        match &self.source {
            StreamSource::YouTube { yt_id } => Some(format!(
                "https://youtube.com/watch?v={}",
                utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET)
            )),
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
