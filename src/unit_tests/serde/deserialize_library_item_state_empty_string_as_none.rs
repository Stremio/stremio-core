use crate::types::library::LibraryItemState;

#[test]
fn deserialize_library_item_state_empty_string_as_none() {
    let library_item_state = LibraryItemState {
        last_watched: None,
        time_watched: 0,
        time_offset: 0,
        overall_time_watched: 0,
        times_watched: 0,
        flagged_watched: 0,
        duration: 0,
        video_id: None,
        watched: None,
        last_vid_released: None,
        no_notif: false,
    };
    let library_item_state_json = r#"
    {
        "lastWatched": "",
        "timeWatched": 0,
        "timeOffset": 0,
        "overallTimeWatched": 0,
        "timesWatched": 0,
        "flaggedWatched": 0,
        "duration": 0,
        "video_id": "",
        "watched": "",
        "lastVidReleased": "",
        "noNotif": false
    }
    "#;
    let library_item_state_deserialize = serde_json::from_str(&library_item_state_json).unwrap();
    assert_eq!(
        library_item_state, library_item_state_deserialize,
        "LibraryItemState deserialized successfully"
    );
}
