use crate::types::resource::{Subtitles, UrlExtended};
use serde_test::{assert_tokens, Token};
use url::Url;

#[test]
fn subtitles() {
    assert_tokens(
        &Subtitles {
            lang: "lang".to_owned(),
            url: UrlExtended::Url(Url::parse("https://url").unwrap()),
        },
        &[
            Token::Struct {
                name: "Subtitles",
                len: 2,
            },
            Token::Str("lang"),
            Token::Str("lang"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
        ],
    );
}
