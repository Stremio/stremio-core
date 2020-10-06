use crate::types::library::LibraryItemState;
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_library_item_state() {
    let library_item_states = vec![
        LibraryItemState {
            last_watched: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            time_watched: 1,
            time_offset: 1,
            overall_time_watched: 1,
            times_watched: 1,
            flagged_watched: 1,
            duration: 1,
            video_id: Some("video_id".to_owned()),
            watched: Some("watched".to_owned()),
            last_vid_released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            no_notif: true,
        },
        LibraryItemState {
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
        },
        LibraryItemState {
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
        },
    ];
    let library_item_states_json = r#"
    [
        {
            "lastWatched": "2020-01-01T00:00:00Z",
            "timeWatched": 1,
            "timeOffset": 1,
            "overallTimeWatched": 1,
            "timesWatched": 1,
            "flaggedWatched": 1,
            "duration": 1,
            "video_id": "video_id",
            "watched": "watched",
            "lastVidReleased": "2020-01-01T00:00:00Z",
            "noNotif": true
        },
        {
            "lastWatched": null,
            "timeWatched": 0,
            "timeOffset": 0,
            "overallTimeWatched": 0,
            "timesWatched": 0,
            "flaggedWatched": 0,
            "duration": 0,
            "video_id": null,
            "watched": null,
            "noNotif": false
        },
        {
            "lastWatched": null,
            "timeWatched": 0,
            "timeOffset": 0,
            "overallTimeWatched": 0,
            "timesWatched": 0,
            "flaggedWatched": 0,
            "duration": 0,
            "video_id": null,
            "watched": null,
            "lastVidReleased": null,
            "noNotif": false
        }
    ]
    "#;
    let library_item_states_deserialize: Vec<LibraryItemState> =
        serde_json::from_str(&library_item_states_json).unwrap();
    assert_eq!(
        library_item_states, library_item_states_deserialize,
        "LibraryItemState deserialized successfully"
    );
}

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
