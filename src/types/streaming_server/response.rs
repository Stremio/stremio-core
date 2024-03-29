use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::streaming_server::Settings;

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
