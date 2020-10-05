use crate::types::library::LibraryItemBehaviorHints;

#[test]
fn deserialize_library_item_behavior_hints() {
    let library_item_behavior_hints = vec![
        LibraryItemBehaviorHints {
            default_video_id: Some("default_video_id".to_owned()),
        },
        LibraryItemBehaviorHints {
            default_video_id: None,
        },
    ];
    let library_item_behavior_hints_json = r#"
    [
        {
            "defaultVideoId": "default_video_id"
        },
        {
            "defaultVideoId": null
        }
    ]
    "#;
    let library_item_behavior_hints_deserialize: Vec<LibraryItemBehaviorHints> =
        serde_json::from_str(&library_item_behavior_hints_json).unwrap();
    assert_eq!(
        library_item_behavior_hints, library_item_behavior_hints_deserialize,
        "LibraryItemBehaviorHints deserialized successfully"
    );
}
