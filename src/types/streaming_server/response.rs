use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::{streaming_server::Settings, torrent::InfoHash};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_server_settings_response_deserialization() {
        let json = serde_json::json!({
            "options": [
                {
                    "id": "localAddonEnabled",
                    "label": "ENABLE_LOCAL_FILES_ADDON",
                    "type": "checkbox"
                },
                {
                    "id": "remoteHttps",
                    "label": "ENABLE_REMOTE_HTTPS_CONN",
                    "type": "select",
                    "class": "https",
                    "icon": true,
                    "selections": [
                        {
                            "name": "Disabled",
                            "val": ""
                        },
                        {
                            "name": "172.18.0.7",
                            "val": "172.18.0.7"
                        }
                    ]
                },
                {
                    "id": "cacheSize",
                    "label": "CACHING",
                    "type": "select",
                    "class": "caching",
                    "icon": true,
                    "selections": [
                        {
                            "name": "no caching",
                            "val": 0
                        },
                        {
                            "name": "2GB",
                            "val": 2147483648_u64
                        },
                        {
                            "name": "5GB",
                            "val": 5368709120_u64
                        },
                        {
                            "name": "10GB",
                            "val": 10737418240_u64
                        },
                        {
                            "name": "∞",
                            "val": null
                        }
                    ]
                }
            ],
            "values": {
                "serverVersion": "4.20.8",
                "appPath": "/root/.stremio-server",
                "cacheRoot": "/root/.stremio-server",
                "cacheSize": 2147483648_u64,
                "btMaxConnections": 55,
                "btHandshakeTimeout": 20000,
                "btRequestTimeout": 4000,
                "btDownloadSpeedSoftLimit": 2621440,
                "btDownloadSpeedHardLimit": 3670016,
                "btMinPeersForStable": 5,
                "remoteHttps": "",
                "localAddonEnabled": false,
                "transcodeHorsepower": 0.75,
                "transcodeMaxBitRate": 0,
                "transcodeConcurrency": 1,
                "transcodeTrackConcurrency": 1,
                "transcodeHardwareAccel": false,
                "transcodeProfile": null,
                "allTranscodeProfiles": [],
                "transcodeMaxWidth": 1920
            },
            "baseUrl": "http://172.18.0.7:11470"
        });

        let _response = serde_json::from_value::<SettingsResponse>(json)
            .expect("Should be able to deserialize settings response");
    }
}
