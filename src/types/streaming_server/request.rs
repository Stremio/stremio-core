use serde::{Deserialize, Serialize};

pub struct ArchiveStreamRequest {
    /// The `rar/create` or `zip/create` key returned in the response
    pub response_key: String,
    pub options: ArchiveStreamOptions,
}

impl ArchiveStreamRequest {
    pub fn to_query_pairs(self) -> Vec<(String, String)> {
        let options = serde_json::to_value(&self.options).expect("should serialize");
        let options_object = options.as_object().expect("Should be an object");

        vec![
            (
                "key".into(),
                // append the length of the options
                // keep in mind that `None` options should always be treated as not-set
                // i.e. should not be serialized
                format!(
                    "{key}{length}",
                    key = self.response_key,
                    length = options_object.len()
                ),
            ),
            ("o".into(), options.to_string()),
        ]
    }
}

/// Server's `rar/stream` and `zip/stream` options of the query.
///
/// Format: `rar/stream?key={create_key}{options length}&o={options_json_string}`
///
/// Where all parameters are url encoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_must_include: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_to_serde_json_value_keys_length() {
        // 0 keys
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: None,
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert!(object.is_empty());
        }

        // only fileIdx
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec![],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(1, object.len(), "Only fileIdx is set");
            assert_eq!(object.keys().next().cloned(), Some("fileIdx".to_string()));
        }

        // both keys are set
        {
            let json_value = serde_json::to_value(ArchiveStreamOptions {
                file_idx: Some(1),
                file_must_include: vec!["fileName".into(), "nameFile".into()],
            })
            .expect("Should serialize to Value");

            let object = json_value.as_object().expect("It is a Map");
            assert_eq!(2, object.len(), "Only fileIdx is set");
        }
    }
}
