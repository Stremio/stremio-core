use crate::types::api::CollectionResponse;
use chrono::prelude::TimeZone;
use chrono::Utc;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn collection_response() {
    assert_ser_tokens(
        &CollectionResponse {
            addons: vec![],
            last_modified: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        },
        &[
            Token::Struct {
                name: "CollectionResponse",
                len: 2,
            },
            Token::Str("addons"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("lastModified"),
            Token::Str("2020-01-01T00:00:00Z"),
            Token::StructEnd,
        ],
    );
}
