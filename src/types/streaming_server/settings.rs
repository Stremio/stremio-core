use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SettingsOptionSelectionValue {
    String(String),
    Number(u64),
    Float(f64),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsOptionSelection {
    pub name: String,
    pub val: Option<SettingsOptionSelectionValue>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsOption {
    pub id: String,
    pub selections: Option<Vec<SettingsOptionSelection>>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsValues {
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub base_url: Url,
    pub values: SettingsValues,
    pub options: Vec<SettingsOption>,
}
