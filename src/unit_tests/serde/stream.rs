use crate::types::resource::{Stream, StreamSource};
use crate::unit_tests::serde::default_tokens_ext::DefaultFlattenTokens;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn stream() {
    assert_tokens(
        &vec![
            Stream {
                source: StreamSource::default(),
                title: Some("title".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                subtitles: vec![],
                behavior_hints: vec![("behavior_hint".to_owned(), serde_json::Value::Bool(true))]
                    .iter()
                    .cloned()
                    .collect(),
            },
            Stream {
                source: StreamSource::default(),
                title: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: vec![].iter().cloned().collect(),
            },
        ],
        &[
            vec![Token::Seq { len: Some(2) }, Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("title"),
                Token::Some,
                Token::Str("title"),
                Token::Str("thumbnail"),
                Token::Some,
                Token::Str("thumbnail"),
                Token::Str("behaviorHints"),
                Token::Map { len: Some(1) },
                Token::Str("behavior_hint"),
                Token::Bool(true),
                Token::MapEnd,
                Token::MapEnd,
                Token::Map { len: None },
            ],
            StreamSource::default_flatten_tokens(),
            vec![Token::MapEnd, Token::SeqEnd],
        ]
        .concat(),
    );
    assert_de_tokens(
        &Stream {
            source: StreamSource::default(),
            title: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: vec![].iter().cloned().collect(),
        },
        &[
            vec![Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("title"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::MapEnd,
            ],
        ]
        .concat(),
    );
}
