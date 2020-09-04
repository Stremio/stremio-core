use crate::types::library::{LibBucket, LibItem, LibItemBehaviorHints, LibItemState};
use crate::types::resource::PosterShape;
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_lib_bucket() {
    let lib_bucket = LibBucket {
        uid: None,
        items: vec![
            (
                "id1".to_owned(),
                LibItem {
                    id: "id1".to_owned(),
                    removed: false,
                    temp: false,
                    ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                    mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                    state: Default::default(),
                    name: "name".to_owned(),
                    type_name: "type_name".to_owned(),
                    poster: None,
                    poster_shape: Default::default(),
                    behavior_hints: Default::default(),
                },
            ),
            (
                "id2".to_owned(),
                LibItem {
                    id: "id2".to_owned(),
                    removed: false,
                    temp: false,
                    ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
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
                    poster_shape: PosterShape::Square,
                    behavior_hints: LibItemBehaviorHints {
                        default_video_id: None,
                    },
                },
            ),
        ]
        .into_iter()
        .collect(),
    };
    let lib_bucket_json = r#"
    {
        "uid": null,
        "items": {
            "id1": {
                "_id": "id1",
                "name": "name",
                "type": "type_name",
                "poster": null,
                "posterShape": "poster",
                "removed": false,
                "temp": false,
                "_ctime": "2020-01-01T00: 00: 00Z",
                "_mtime": "2020-01-01T00: 00: 00Z",
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
            },
            "id2": {
                "_id": "id2",
                "name": "name",
                "type": "type_name",
                "poster": null,
                "posterShape": "square",
                "removed": false,
                "temp": false,
                "_ctime": "2020-01-01T00: 00: 00Z",
                "_mtime": "2020-01-01T00: 00: 00Z",
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
        }
    }
    "#;
    let lib_bucket_deserialize = serde_json::from_str(&lib_bucket_json).unwrap();
    assert_eq!(
        lib_bucket, lib_bucket_deserialize,
        "library bucket deserialized successfully"
    );
}

#[test]
fn deserialize_lib_bucket_with_user() {
    let lib_bucket = LibBucket {
        uid: Some("user_id".to_owned()),
        items: vec![
            (
                "id1".to_owned(),
                LibItem {
                    id: "id1".to_owned(),
                    removed: false,
                    temp: false,
                    ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                    mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                    state: Default::default(),
                    name: "name".to_owned(),
                    type_name: "type_name".to_owned(),
                    poster: None,
                    poster_shape: Default::default(),
                    behavior_hints: Default::default(),
                },
            ),
            (
                "id2".to_owned(),
                LibItem {
                    id: "id2".to_owned(),
                    removed: false,
                    temp: false,
                    ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
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
                    poster_shape: PosterShape::Square,
                    behavior_hints: LibItemBehaviorHints {
                        default_video_id: None,
                    },
                },
            ),
        ]
        .into_iter()
        .collect(),
    };
    let lib_bucket_json = r#"
    {
        "uid": "user_id",
        "items": {
            "id1": {
                "_id": "id1",
                "name": "name",
                "type": "type_name",
                "poster": null,
                "posterShape": "poster",
                "removed": false,
                "temp": false,
                "_ctime": "2020-01-01T00:00:00Z",
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
            },
            "id2": {
                "_id": "id2",
                "name": "name",
                "type": "type_name",
                "poster": null,
                "posterShape": "square",
                "removed": false,
                "temp": false,
                "_ctime": "2020-01-01T00:00:00Z",
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
        }
    }
    "#;
    let lib_bucket_deserialize = serde_json::from_str(&lib_bucket_json).unwrap();
    assert_eq!(
        lib_bucket, lib_bucket_deserialize,
        "library bucket deserialized successfully"
    );
}
