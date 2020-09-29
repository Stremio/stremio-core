use crate::types::library::LibBucket;

#[test]
fn deserialize_lib_bucket() {
    let lib_bucket = LibBucket {
        uid: Some("uid".to_owned()),
        items: vec![].into_iter().collect(),
    };
    let lib_bucket_json = r#"
    {
        "uid": "uid",
        "items": {}
    }
    "#;
    let lib_bucket_deserialize = serde_json::from_str(&lib_bucket_json).unwrap();
    assert_eq!(
        lib_bucket, lib_bucket_deserialize,
        "LibBucket deserialized successfully"
    );
}
