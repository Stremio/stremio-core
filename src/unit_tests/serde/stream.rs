use crate::types::resource::{Stream, StreamBehaviorHints, StreamSource, Subtitles};
use crate::unit_tests::serde::default_tokens_ext::{DefaultFlattenTokens, DefaultTokens};
use serde_test::{assert_de_tokens, assert_tokens, Configure, Token};

#[test]
fn stream() {
    assert_tokens(
        &vec![
            Stream {
                source: StreamSource::default(),
                name: None,
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: StreamBehaviorHints::default(),
            },
            Stream {
                source: StreamSource::default(),
                name: Some("name".to_owned()),
                description: Some("description".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                subtitles: vec![Subtitles::default()],
                behavior_hints: StreamBehaviorHints {
                    not_web_ready: true,
                    ..StreamBehaviorHints::default()
                },
            },
        ]
        .readable(),
        &[
            vec![Token::Seq { len: Some(2) }],
            vec![Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![Token::MapEnd],
            vec![Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("name"),
                Token::Some,
                Token::Str("name"),
                Token::Str("description"),
                Token::Some,
                Token::Str("description"),
                Token::Str("thumbnail"),
                Token::Some,
                Token::Str("thumbnail"),
                Token::Str("subtitles"),
                Token::Some,
                Token::Seq { len: Some(1) },
            ],
            Subtitles::default_tokens(),
            vec![
                Token::SeqEnd,
                Token::Str("behaviorHints"),
                Token::Some,
                Token::Map { len: None },
                Token::Str("notWebReady"),
                Token::Bool(true),
                Token::MapEnd,
                Token::MapEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
    assert_de_tokens(
        &Stream {
            source: StreamSource::default(),
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        }
        .readable(),
        &[
            vec![Token::Map { len: None }],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("name"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("subtitles"),
                Token::None,
                Token::Str("behaviorHints"),
                Token::None,
            ],
            // StreamBehaviorHints::default_tokens(),
            vec![Token::MapEnd],
        ]
        .concat(),
    );
}
