use crate::types::library::LibraryItemBehaviorHints;
use serde_test::{assert_tokens, Token};

#[test]
fn library_item_behavior_hints() {
    assert_tokens(
        &vec![
            LibraryItemBehaviorHints {
                default_video_id: Some("default_video_id".to_owned()),
                other: Default::default(),
            },
            LibraryItemBehaviorHints {
                default_video_id: None,
                other: Default::default(),
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Map { len: None },
            Token::Str("defaultVideoId"),
            Token::Some,
            Token::Str("default_video_id"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("defaultVideoId"),
            Token::None,
            Token::MapEnd,
            Token::SeqEnd,
        ],
    );
}
