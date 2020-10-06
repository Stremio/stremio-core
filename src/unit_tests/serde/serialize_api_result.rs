use crate::types::api::{APIError, APIResult};

#[test]
fn serialize_api_result_err() {
    let error = APIResult::<()>::Err {
        error: APIError {
            message: "message".to_owned(),
            code: 1,
        },
    };
    let error_json = r#"{"error":{"message":"message","code":1}}"#;
    let error_serialize = serde_json::to_string(&error).unwrap();
    assert_eq!(error_json, error_serialize, "Err serialized successfully");
}

#[test]
fn serialize_api_result_ok() {
    let ok = APIResult::<()>::Ok { result: () };
    let ok_json = r#"{"result":null}"#;
    let ok_serialize = serde_json::to_string(&ok).unwrap();
    assert_eq!(ok_json, ok_serialize, "Ok serialized successfully");
}
