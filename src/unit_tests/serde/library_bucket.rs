use crate::types::library::LibraryBucket;
use serde_test::{assert_tokens, Token};
use std::collections::HashMap;

#[test]
fn library_bucket() {
    assert_tokens(
        &vec![
            LibraryBucket {
                uid: Some("uid".into()),
                items: HashMap::new(),
            },
            LibraryBucket {
                uid: None,
                items: HashMap::new(),
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "LibraryBucket",
                len: 2,
            },
            Token::Str("uid"),
            Token::Some,
            Token::Str("uid"),
            Token::Str("items"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::StructEnd,
            Token::Struct {
                name: "LibraryBucket",
                len: 2,
            },
            Token::Str("uid"),
            Token::None,
            Token::Str("items"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
}
