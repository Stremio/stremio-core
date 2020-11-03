use crate::types::library::{LibraryItem, LibraryItemBehaviorHints, LibraryItemState};
use crate::types::resource::PosterShape;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use chrono::prelude::TimeZone;
use chrono::Utc;
use serde_test::{assert_de_tokens, Token};

#[test]
fn library_item_empty_string_as_none() {
    assert_de_tokens(
        &LibraryItem {
            id: "id".to_owned(),
            name: "name".to_owned(),
            r#type: "type".to_owned(),
            poster: None,
            poster_shape: PosterShape::default(),
            removed: false,
            temp: false,
            ctime: None,
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: LibraryItemState::default(),
            behavior_hints: LibraryItemBehaviorHints::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "LibraryItem",
                    len: 11,
                },
                Token::Str("_id"),
                Token::Str("id"),
                Token::Str("name"),
                Token::Str("name"),
                Token::Str("type"),
                Token::Str("type"),
                Token::Str("poster"),
                Token::Some,
                Token::Str(""),
                Token::Str("posterShape"),
            ],
            PosterShape::default_tokens(),
            vec![
                Token::Str("removed"),
                Token::Bool(false),
                Token::Str("temp"),
                Token::Bool(false),
                Token::Str("_ctime"),
                Token::Some,
                Token::Str(""),
                Token::Str("_mtime"),
                Token::Str("2020-01-01T00:00:00Z"),
                Token::Str("state"),
            ],
            LibraryItemState::default_tokens(),
            vec![Token::Str("behaviorHints")],
            LibraryItemBehaviorHints::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
