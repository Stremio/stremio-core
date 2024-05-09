use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::{streaming_server::Settings, torrent::InfoHash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "experimental")]
pub struct ArchiveCreateResponse {
    pub key: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
    pub base_url: Url,
    pub values: Settings,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetHTTPSResponse {
    pub ip_address: String,
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpensubtitlesParamsResponse {
    pub hash: InfoHash,
    pub size: u64,
}
