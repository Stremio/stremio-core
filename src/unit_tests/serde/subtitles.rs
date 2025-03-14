use crate::types::resource::Subtitles;
use serde_test::{assert_tokens, Token};
use url::Url;

#[test]
fn subtitles() {
    assert_tokens(
        &Subtitles {
            id: "id".into(),
            lang: "lang".to_owned(),
            url: Url::parse("https://url").unwrap(),
        },
        &[
            Token::Struct {
                name: "Subtitles",
                len: 3,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("lang"),
            Token::Str("lang"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
        ],
    );
}
