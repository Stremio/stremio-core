pub mod empty_string_as_null {
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
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(s) if !s.is_empty() => Ok(Some(s)),
            _ => Ok(None),
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

            let null_json = serde_json::json!({
                "field":null,
            });

            assert_eq!(
                test_struct,
                serde_json::from_value::<TestStruct>(empty_string.clone()).expect("Valid Json")
            );
            assert_eq!(
                empty_string,
                serde_json::to_value(&test_struct).expect("Should serialize")
            );
            assert_eq!(
                test_struct,
                serde_json::from_value::<TestStruct>(null_json.clone()).expect("Should serialize")
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
