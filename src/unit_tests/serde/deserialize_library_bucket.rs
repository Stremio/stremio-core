use crate::types::library::LibraryBucket;

#[test]
fn deserialize_library_bucket() {
    let library_bucket = LibraryBucket {
        uid: Some("uid".to_owned()),
        items: vec![].into_iter().collect(),
    };
    let library_bucket_json = r#"
    {
        "uid": "uid",
        "items": {}
    }
    "#;
    let library_bucket_deserialize = serde_json::from_str(&library_bucket_json).unwrap();
    assert_eq!(
        library_bucket, library_bucket_deserialize,
        "LibraryBucket deserialized successfully"
    );
}
