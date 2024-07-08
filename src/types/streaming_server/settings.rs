use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError};

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
    pub proxy_streams_enabled: bool,
    pub bt_max_connections: u64,
    pub bt_handshake_timeout: u64,
    pub bt_request_timeout: u64,
    pub bt_download_speed_soft_limit: f64,
    pub bt_download_speed_hard_limit: f64,
    pub bt_min_peers_for_stable: u64,
}
mod empty_string_as_null {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *value {
            None => serializer.serialize_str(""),
            Some(ref s) => serializer.serialize_str(s),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::streaming_server::Settings;

    #[test]
    fn test_serialize_remote_https_none() {
        let settings = Settings {
            app_path: "app_path".to_string(),
            cache_root: "cache_root".to_string(),
            server_version: "server_version".to_string(),
            remote_https: None,
            transcode_profile: None,
            cache_size: None,
            proxy_streams_enabled: true,
            bt_max_connections: 10,
            bt_handshake_timeout: 30,
            bt_request_timeout: 60,
            bt_download_speed_soft_limit: 100.0,
            bt_download_speed_hard_limit: 200.0,
            bt_min_peers_for_stable: 5,
        };

        let json = serde_json::to_string(&settings).unwrap();

        println!("JSON: {}", json);

        assert_eq!(
            json,
            r#"{"appPath":"app_path","cacheRoot":"cache_root","serverVersion":"server_version","remoteHttps":"","transcodeProfile":null,"cacheSize":null,"proxyStreamsEnabled":true,"btMaxConnections":10,"btHandshakeTimeout":30,"btRequestTimeout":60,"btDownloadSpeedSoftLimit":100.0,"btDownloadSpeedHardLimit":200.0,"btMinPeersForStable":5}"#
        );
    }

    #[test]
    fn test_serialize_remote_https_some() {
        let settings = Settings {
            app_path: "app_path".to_string(),
            cache_root: "cache_root".to_string(),
            server_version: "server_version".to_string(),
            remote_https: Some("https://example.com".to_string()),
            transcode_profile: None,
            cache_size: None,
            proxy_streams_enabled: true,
            bt_max_connections: 10,
            bt_handshake_timeout: 30,
            bt_request_timeout: 60,
            bt_download_speed_soft_limit: 100.0,
            bt_download_speed_hard_limit: 200.0,
            bt_min_peers_for_stable: 5,
        };

        let json = serde_json::to_string(&settings).unwrap();

        println!("JSON: {}", json);

        assert_eq!(
            json,
            r#"{"appPath":"app_path","cacheRoot":"cache_root","serverVersion":"server_version","remoteHttps":"https://example.com","transcodeProfile":null,"cacheSize":null,"proxyStreamsEnabled":true,"btMaxConnections":10,"btHandshakeTimeout":30,"btRequestTimeout":60,"btDownloadSpeedSoftLimit":100.0,"btDownloadSpeedHardLimit":200.0,"btMinPeersForStable":5}"#
        );
    }

    #[test]

    fn test_deserialize_remote_https_none() {
        let json = r#"{"appPath":"app_path","cacheRoot":"cache_root","serverVersion":"server_version","remoteHttps":"","transcodeProfile":null,"cacheSize":null,"proxyStreamsEnabled":true,"btMaxConnections":10,"btHandshakeTimeout":30,"btRequestTimeout":60,"btDownloadSpeedSoftLimit":100.0,"btDownloadSpeedHardLimit":200.0,"btMinPeersForStable":5}"#;

        let settings: Settings = serde_json::from_str(json).unwrap();

        assert_eq!(settings.remote_https, None);
    }

    #[test]

    fn test_deserialize_remote_https_some() {
        let json = r#"{"appPath":"app_path","cacheRoot":"cache_root","serverVersion":"server_version","remoteHttps":"https://example.com","transcodeProfile":null,"cacheSize":null,"proxyStreamsEnabled":true,"btMaxConnections":10,"btHandshakeTimeout":30,"btRequestTimeout":60,"btDownloadSpeedSoftLimit":100.0,"btDownloadSpeedHardLimit":200.0,"btMinPeersForStable":5}"#;

        let settings: Settings = serde_json::from_str(json).unwrap();

        assert_eq!(
            settings.remote_https,
            Some("https://example.com".to_string())
        );
    }
}
