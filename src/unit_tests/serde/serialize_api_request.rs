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
        "Logout serialized successfully"
    );
}

#[test]
fn serialize_api_request_addon_collection_get() {
    let addon_collection_get = APIRequest::AddonCollectionGet {
        auth_key: "auth_key".to_owned(),
        update: true,
    };
    let addon_collection_get_json =
        r#"{"type":"AddonCollectionGet","authKey":"auth_key","update":true}"#;
    let addon_collection_get_serialize = serde_json::to_string(&addon_collection_get).unwrap();
    assert_eq!(
        addon_collection_get_json, addon_collection_get_serialize,
        "AddonCollectionGet serialized successfully"
    );
}
