use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub app_path: String,
    pub cache_root: String,
    pub server_version: String,
    pub remote_https: Option<String>,
    pub cache_size: Option<f64>,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
}
