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
            label: None,
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

#[test]
fn subtitles_with_label() {
    assert_tokens(
        &Subtitles {
            id: "id".into(),
            lang: "eng".to_owned(),
            url: Url::parse("https://url").unwrap(),
            label: Some("eng #1 [opensubtitles] 1080p.BluRay".to_owned()),
        },
        &[
            Token::Struct {
                name: "Subtitles",
                len: 4,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("lang"),
            Token::Str("eng"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::Str("label"),
            Token::Some,
            Token::Str("eng #1 [opensubtitles] 1080p.BluRay"),
            Token::StructEnd,
        ],
    );
}
