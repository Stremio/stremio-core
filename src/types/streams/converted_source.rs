use url::Url;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};

use crate::types::{
    resource::{deserialize_stream_source_external, StreamSource},
    torrent::InfoHash,
};

/// Trait which defines the StreamSource state data structures in Core.
pub trait StreamSourceTrait: sealed::Sealed {}

impl StreamSourceTrait for ConvertedStreamSource {}
impl sealed::Sealed for ConvertedStreamSource {}

impl sealed::Sealed for StreamSource {}
impl StreamSourceTrait for StreamSource {}

/// only we should be able to define which data structures are StreamSource states!
mod sealed {
    pub trait Sealed {}
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(Default))]
#[serde(untagged)]
pub enum ConvertedStreamSource {
    Url {
        url: Url,
    },
    #[cfg_attr(test, derivative(Default))]
    #[serde(rename_all = "camelCase")]
    YouTube {
        /// The Streaming url
        url: Url,
        yt_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Torrent {
        /// The Streaming url
        url: Url,
        info_hash: InfoHash,
        #[serde(default)]
        file_idx: Option<u16>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, alias = "sources")]
        announce: Vec<String>,
        #[serde_as(deserialize_as = "DefaultOnNull")]
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        file_must_include: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    PlayerFrame {
        /// The Streaming url
        player_frame_url: Url,
    },
    #[serde(
        rename_all = "camelCase",
        deserialize_with = "deserialize_stream_source_external"
    )]
    External {
        /// The Streaming url
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
