use crate::types::resource::{Stream, StreamBehaviorHints, StreamSource, Subtitles};
use crate::unit_tests::serde::default_tokens_ext::{DefaultFlattenTokens, DefaultTokens};
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn stream() {
    assert_tokens(
        &vec![
            Stream {
                source: StreamSource::default(),
                title: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: StreamBehaviorHints::default(),
            },
            Stream {
                source: StreamSource::default(),
                title: Some("title".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                subtitles: vec![Subtitles::default()],
                behavior_hints: StreamBehaviorHints {
                    not_web_ready: true,
                    ..StreamBehaviorHints::default()
                },
            },
        ],
        &[
            vec![Token::Seq { len: Some(2) }, Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![Token::MapEnd, Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("title"),
                Token::Some,
                Token::Str("title"),
                Token::Str("thumbnail"),
                Token::Some,
                Token::Str("thumbnail"),
                Token::Str("subtitles"),
                Token::Seq { len: Some(1) },
            ],
            Subtitles::default_tokens(),
            vec![
                Token::SeqEnd,
                Token::Str("behaviorHints"),
                Token::Struct {
                    name: "StreamBehaviorHints",
                    len: 1,
                },
                Token::Str("notWebReady"),
                Token::Bool(true),
                Token::StructEnd,
                Token::MapEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
    assert_de_tokens(
        &Stream {
            source: StreamSource::default(),
            title: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
        &[
            vec![Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("title"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("subtitles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            StreamBehaviorHints::default_tokens(),
            vec![Token::MapEnd],
        ]
        .concat(),
    );
}
