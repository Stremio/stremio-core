use crate::types::library::{LibItem, LibItemBehaviorHints, LibItemState};
use crate::types::resource::PosterShape;
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_lib_item() {
    let lib_items = vec![
        // ALL fields are defined with SOME value
        LibItem {
            id: "id1".to_owned(),
            removed: true,
            temp: true,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: LibItemState {
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
            name: "name".to_owned(),
            type_name: "type_name".to_owned(),
            poster: Some("poster".to_owned()),
            poster_shape: PosterShape::Square,
            behavior_hints: LibItemBehaviorHints {
                default_video_id: Some("default_video_id".to_owned()),
            },
        },
        // serde(default) are omited
        LibItem {
            id: "id2".to_owned(),
            removed: false,
            temp: false,
            ctime: None,
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: LibItemState {
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
            name: "name".to_owned(),
            type_name: "type_name".to_owned(),
            poster: None,
            poster_shape: PosterShape::Poster,
            behavior_hints: LibItemBehaviorHints {
                default_video_id: None,
            },
        },
        // ALL NONEs are set to null.
        // poster shape is invalid
        LibItem {
            id: "id3".to_owned(),
            removed: false,
            temp: false,
            ctime: None,
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: LibItemState {
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
            name: "name".to_owned(),
            type_name: "type_name".to_owned(),
            poster: None,
            poster_shape: PosterShape::Poster,
            behavior_hints: LibItemBehaviorHints {
                default_video_id: None,
            },
        },
    ];
    let lib_items_json = r#"
    [
        {
            "_id": "id1",
            "name": "name",
            "type": "type_name",
            "poster": "poster",
            "posterShape": "square",
            "removed": true,
            "temp": true,
            "_ctime": "2020-01-01T00:00:00Z",
            "_mtime": "2020-01-01T00:00:00Z",
            "state": {
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
            "behaviorHints": {
                "defaultVideoId": "default_video_id"
            }
        },
        {
            "_id": "id2",
            "name": "name",
            "type": "type_name",
            "removed": false,
            "temp": false,
            "_mtime": "2020-01-01T00:00:00Z",
            "state": {
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
            }
        },
        {
            "_id": "id3",
            "name": "name",
            "type": "type_name",
            "poster": null,
            "posterShape": "invalid",
            "removed": false,
            "temp": false,
            "_ctime": null,
            "_mtime": "2020-01-01T00:00:00Z",
            "state": {
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
            },
            "behaviorHints": {
                "defaultVideoId": null
            }
        }
    ]
    "#;
    let lib_items_deserialize: Vec<LibItem> = serde_json::from_str(&lib_items_json).unwrap();
    assert_eq!(
        lib_items, lib_items_deserialize,
        "LibItem deserialized successfully"
    );
}

#[test]
fn deserialize_lib_item_empty_string_as_none() {
    let lib_item = LibItem {
        id: "id1".to_owned(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: LibItemState {
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
        name: "name".to_owned(),
        type_name: "type_name".to_owned(),
        poster: None,
        poster_shape: PosterShape::Poster,
        behavior_hints: LibItemBehaviorHints {
            default_video_id: None,
        },
    };
    let lib_item_json = r#"
    {
        "_id": "id1",
        "name": "name",
        "type": "type_name",
        "poster": "",
        "posterShape": "poster",
        "removed": false,
        "temp": false,
        "_ctime": "",
        "_mtime": "2020-01-01T00:00:00Z",
        "state": {
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
        },
        "behaviorHints": {
            "defaultVideoId": null
        }
    }
    "#;
    let lib_item_deserialize = serde_json::from_str(&lib_item_json).unwrap();
    assert_eq!(
        lib_item, lib_item_deserialize,
        "LibItem deserialized successfully"
    );
}
