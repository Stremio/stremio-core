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
    #[cfg(test)]
    mod test {
        use serde::{Deserialize, Serialize};
        #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
        struct TestStruct {
            #[serde(with = "super")]
            field: Option<String>,
        }
        #[test]
        fn test_empty_field() {
            let empty_string = serde_json::json!({
                "field": ""
            });

            let test_struct = TestStruct { field: None };

            assert_eq!(
                test_struct,
                serde_json::from_value::<TestStruct>(empty_string.clone()).expect("Valid Json")
            );
            assert_eq!(
                empty_string,
                serde_json::to_value(test_struct).expect("Should serialize")
            );
        }
        #[test]
        fn test_field() {
            let string = serde_json::json!({
                "field": "https://example.com"
            });

            let test_struct = TestStruct {
                field: Some("https://example.com".to_string()),
            };

            assert_eq!(
                test_struct,
                serde_json::from_value::<TestStruct>(string.clone()).expect("Valid Json")
            );
            assert_eq!(
                string,
                serde_json::to_value(test_struct).expect("Should serialize")
            );
        }
    }
}
