use crate::types::library::LibraryBucketRef;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn library_bucket_ref() {
    assert_ser_tokens(
        &LibraryBucketRef {
            uid: &Some("uid".into()),
            items: [].iter().cloned().collect(),
        },
        &[
            Token::Struct {
                name: "LibraryBucketRef",
                len: 2,
            },
            Token::Str("uid"),
            Token::Some,
            Token::Str("uid"),
            Token::Str("items"),
            Token::Map { len: Some(0) },
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}
