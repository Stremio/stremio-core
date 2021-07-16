use crate::types::resource::SeriesInfo;
use serde_test::{assert_tokens, Token};

#[test]
fn series_info() {
    assert_tokens(
        &SeriesInfo {
            season: 1,
            episode: 1,
        },
        &[
            Token::Struct {
                name: "SeriesInfo",
                len: 2,
            },
            Token::Str("season"),
            Token::U32(1),
            Token::Str("episode"),
            Token::U32(1),
            Token::StructEnd,
        ],
    );
}
