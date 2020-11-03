use crate::types::library::LibraryItemState;
use serde_test::{assert_de_tokens, Token};

#[test]
fn library_item_state_empty_string_as_none() {
    assert_de_tokens(
        &LibraryItemState {
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
        &[
            Token::Struct {
                name: "LibraryItemState",
                len: 11,
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
            Token::Str("watched"),
            Token::Some,
            Token::Str(""),
            Token::Str("lastVidReleased"),
            Token::Some,
            Token::Str(""),
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
        ],
    );
}
