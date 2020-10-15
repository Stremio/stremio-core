use crate::types::addon::ManifestPreview;
use crate::types::library::{LibraryItemBehaviorHints, LibraryItemState};
use crate::types::resource::{MetaItemBehaviorHints, PosterShape};
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

impl DefaultTokens for MetaItemBehaviorHints {
    fn default_token() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "MetaItemBehaviorHints",
                len: 3,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::Str("featuredVideoId"),
            Token::None,
            Token::Str("hasScheduledVideos"),
            Token::Bool(false),
            Token::StructEnd,
        ]
    }
}

impl DefaultTokens for PosterShape {
    fn default_token() -> Vec<Token> {
        vec![
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("poster"),
            Token::Unit,
        ]
    }
}

impl DefaultTokens for ManifestPreview {
    fn default_token() -> Vec<Token> {
        vec![
            Token::Struct {
                name: "ManifestPreview",
                len: 7,
            },
            Token::Str("id"),
            Token::Str(""),
            Token::Str("version"),
            Token::Str("0.0.1"),
            Token::Str("name"),
            Token::Str(""),
            Token::Str("description"),
            Token::None,
            Token::Str("logo"),
            Token::None,
            Token::Str("background"),
            Token::None,
            Token::Str("types"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ]
    }
}
