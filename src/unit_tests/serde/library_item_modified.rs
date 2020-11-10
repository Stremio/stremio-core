use crate::types::api::LibraryItemModified;
use chrono::prelude::TimeZone;
use chrono::Utc;
use serde_test::{assert_de_tokens, Token};

#[test]
fn library_item_modified() {
    assert_de_tokens(
        &LibraryItemModified("".to_owned(), Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        &[
            Token::Tuple { len: 2 },
            Token::Str(""),
            Token::U64(1577836800000),
            Token::TupleEnd,
        ],
    );
}
