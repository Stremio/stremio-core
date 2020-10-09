use crate::types::library::{LibraryItemBehaviorHints, LibraryItemState};
use serde_test::Token;

pub trait DefaultTokens {
    fn default_token() -> Vec<Token>;
}

impl DefaultTokens for LibraryItemState {
    fn default_token() -> Vec<Token> {
        vec![
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
            Token::Str("watched"),
            Token::None,
            Token::Str("lastVidReleased"),
            Token::None,
            Token::Str("noNotif"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for LibraryItemBehaviorHints {
    fn default_token() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "LibraryItemBehaviorHints",
                len: 1,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::StructEnd,
        ]
    }
}
