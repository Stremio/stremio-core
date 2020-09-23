use crate::types::api::{APIError, APIResult};

#[test]
fn serialize_api_result_err() {
    let error = APIResult::<String>::Err {
        error: APIError {
            message: "message".to_owned(),
            code: 1,
        },
    };
    let error_json = r#"{"error":{"message":"message","code":1}}"#;
    let error_serialize = serde_json::to_string(&error).unwrap();
    assert_eq!(error_json, error_serialize, "Err serialized successfully");
}
