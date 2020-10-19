use crate::types::library::LibraryItemBehaviorHints;
use serde_test::{assert_tokens, Token};

#[test]
fn library_item_behavior_hints() {
    assert_tokens(
        &vec![
            LibraryItemBehaviorHints {
                default_video_id: Some("default_video_id".to_owned()),
            },
            LibraryItemBehaviorHints {
                default_video_id: None,
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "LibraryItemBehaviorHints",
                len: 1,
            },
            Token::Str("defaultVideoId"),
            Token::Some,
            Token::Str("default_video_id"),
            Token::StructEnd,
            Token::Struct {
                name: "LibraryItemBehaviorHints",
                len: 1,
            },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
}
