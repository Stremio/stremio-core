use crate::types::resource::MetaItemBehaviorHints;

#[test]
fn deserialize_meta_item_behavior_hints() {
    let meta_item_behavior_hints = vec![
        MetaItemBehaviorHints {
            default_video_id: Some("default_video_id".to_owned()),
            featured_video_id: Some("featured_video_id".to_owned()),
            has_scheduled_videos: true,
        },
        MetaItemBehaviorHints {
            default_video_id: None,
            featured_video_id: None,
            has_scheduled_videos: false,
        },
    ];
    let meta_item_behavior_hints_json = r#"
    [
        {
            "defaultVideoId": "default_video_id",
            "featuredVideoId": "featured_video_id",
            "hasScheduledVideos": true
        },
        {
            "defaultVideoId": null,
            "featuredVideoId": null
        }
    ]
    "#;
    let meta_item_behavior_hints_deserialize: Vec<MetaItemBehaviorHints> =
        serde_json::from_str(&meta_item_behavior_hints_json).unwrap();
    assert_eq!(
        meta_item_behavior_hints, meta_item_behavior_hints_deserialize,
        "MetaItemBehaviorHints deserialized successfully"
    );
}
