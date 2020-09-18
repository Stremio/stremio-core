use crate::types::api::{APIRequest, AuthRequest, GDPRConsentWithTime};
use crate::types::profile::GDPRConsent;
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn serialize_api_request_auth() {
    let auth = vec![
        APIRequest::Auth(AuthRequest::Login {
            email: "email".to_owned(),
            password: "password".to_owned(),
        }),
        APIRequest::Auth(AuthRequest::Register {
            email: "email".to_owned(),
            password: "password".to_owned(),
            gdpr_consent: GDPRConsentWithTime {
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: true,
                    from: "from".to_owned(),
                },
                time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            },
        }),
    ];
    let auth_json = r#"[{"type":"Auth","type":"Login","email":"email","password":"password"},{"type":"Auth","type":"Register","email":"email","password":"password","gdpr_consent":{"tos":true,"privacy":true,"marketing":true,"from":"from","time":"2020-01-01T00:00:00Z"}}]"#;
    let auth_serialize = serde_json::to_string(&auth).unwrap();
    assert_eq!(auth_json, auth_serialize, "Auth serialized successfully");
}

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

#[test]
fn serialize_api_request_addon_collection_set() {
    let addon_collection_set = APIRequest::AddonCollectionSet {
        auth_key: "auth_key".to_owned(),
        addons: vec![],
    };
    let addon_collection_set_json =
        r#"{"type":"AddonCollectionSet","authKey":"auth_key","addons":[]}"#;
    let addon_collection_set_serialize = serde_json::to_string(&addon_collection_set).unwrap();
    assert_eq!(
        addon_collection_set_json, addon_collection_set_serialize,
        "AddonCollectionSet serialized successfully"
    );
}
