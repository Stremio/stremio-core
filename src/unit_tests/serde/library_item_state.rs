use crate::types::library::LibraryItemState;
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn library_item_state() {
    assert_tokens(
        &vec![
            LibraryItemState {
                last_watched: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                time_watched: 1,
                time_offset: 1,
                overall_time_watched: 1,
                times_watched: 1,
                flagged_watched: 1,
                duration: 1,
                video_id: Some("tt2934286:1:5".to_owned()),
                stream: Some("eAEBXgGh%2FnsidXJsIjo".to_owned()),
                watched: Some("tt2934286:1:5:5:eJyTZwAAAEAAIA==".parse().unwrap()),
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
                stream: None,
                watched: None,
                no_notif: false,
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "LibraryItemState",
                len: 11,
            },
            Token::Str("lastWatched"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::Str("timeWatched"),
            Token::U64(1),
            Token::Str("timeOffset"),
            Token::U64(1),
            Token::Str("overallTimeWatched"),
            Token::U64(1),
            Token::Str("timesWatched"),
            Token::U32(1),
            Token::Str("flaggedWatched"),
            Token::U32(1),
            Token::Str("duration"),
            Token::U64(1),
            Token::Str("video_id"),
            Token::Some,
            Token::Str("tt2934286:1:5"),
            Token::Str("stream"),
            Token::Some,
            Token::Str("eAEBXgGh%2FnsidXJsIjo"),
            Token::Str("watched"),
            Token::Some,
            Token::Str("tt2934286:1:5:5:eJyTZwAAAEAAIA=="),
            Token::Str("noNotif"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Struct {
                name: "LibraryItemState",
                len: 11,
            },
            Token::Str("lastWatched"),
            Token::None,
            Token::Str("timeWatched"),
            Token::U64(0),
            Token::Str("timeOffset"),
            Token::U64(0),
            Token::Str("overallTimeWatched"),
            Token::U64(0),
            Token::Str("timesWatched"),
            Token::U32(0),
            Token::Str("flaggedWatched"),
            Token::U32(0),
            Token::Str("duration"),
            Token::U64(0),
            Token::Str("video_id"),
            Token::None,
            Token::Str("stream"),
            Token::None,
            Token::Str("watched"),
            Token::None,
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &vec![
            LibraryItemState {
                last_watched: None,
                time_watched: 0,
                time_offset: 0,
                overall_time_watched: 0,
                times_watched: 0,
                flagged_watched: 0,
                duration: 0,
                video_id: None,
                stream: None,
                watched: None,
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
                stream: None,
                watched: None,
                no_notif: false,
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "LibraryItemState",
                len: 7,
            },
            Token::Str("timeWatched"),
            Token::U64(0),
            Token::Str("timeOffset"),
            Token::U64(0),
            Token::Str("overallTimeWatched"),
            Token::U64(0),
            Token::Str("timesWatched"),
            Token::U32(0),
            Token::Str("flaggedWatched"),
            Token::U32(0),
            Token::Str("duration"),
            Token::U64(0),
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
            Token::Struct {
                name: "LibraryItemState",
                len: 10,
            },
            Token::Str("lastWatched"),
            Token::Some,
            Token::Str(""),
            Token::Str("timeWatched"),
            Token::U64(0),
            Token::Str("timeOffset"),
            Token::U64(0),
            Token::Str("overallTimeWatched"),
            Token::U64(0),
            Token::Str("timesWatched"),
            Token::U64(0),
            Token::Str("flaggedWatched"),
            Token::U64(0),
            Token::Str("duration"),
            Token::U64(0),
            Token::Str("video_id"),
            Token::Some,
            Token::Str(""),
            Token::Str("stream"),
            Token::Some,
            Token::Str(""),
            Token::Str("watched"),
            Token::Some,
            Token::Str(""),
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
}
