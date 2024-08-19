use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError};

use crate::types::serde_ext::empty_string_as_null;

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub app_path: String,
    pub cache_root: String,
    pub server_version: String,
    #[serde(with = "empty_string_as_null")]
    pub remote_https: Option<String>,
    // Set to default in case the value is anything other than a string or null
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub transcode_profile: Option<String>,
    pub cache_size: Option<f64>,
    #[serde(default)]
    pub proxy_streams_enabled: bool,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
}
