use crate::constants::YOUTUBE_ADDON_ID_PREFIX;
use crate::types::resource::Subtitles;
use boolinator::Boolinator;
#[cfg(test)]
use derivative::Derivative;
use magnet_url::Magnet;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, Strict};
use std::collections::HashMap;
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
        #[serde(default)]
        announce: Vec<String>,
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

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct StreamBehaviorHints {
    #[serde(default, skip_serializing_if = "is_default_value")]
    pub not_web_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binge_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_whitelist: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

fn is_default_value<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
