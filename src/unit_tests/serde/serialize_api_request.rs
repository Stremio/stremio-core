use crate::types::api::APIRequest;

#[test]
fn serialize_api_request_logout() {
    let logout = APIRequest::Logout {
        auth_key: "auth_key".to_owned(),
    };
    let logout_json = r#"{"type":"Logout","authKey":"auth_key"}"#;
    let logout_serialize = serde_json::to_string(&logout).unwrap();
    assert_eq!(
        logout_json, logout_serialize,
        "logout serialized successfully"
    );
}
