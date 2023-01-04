use crate::types::api::LibraryItemModified;
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, Token};

#[test]
fn library_item_modified() {
    assert_de_tokens(
        &LibraryItemModified(
            "".to_owned(),
            Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        ),
        &[
            Token::Tuple { len: 2 },
            Token::Str(""),
            Token::U64(1577836800000),
            Token::TupleEnd,
        ],
    );
}
