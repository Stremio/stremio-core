use serde_test::{assert_de_tokens, assert_tokens, Configure, Token};

use crate::{
    types::{
        addon::{ResourceResponse, ResourceResponseCache},
        resource::{MetaItem, Stream, StreamBehaviorHints, StreamSource},
    },
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};

#[test]
fn resource_response() {
    assert_tokens(
        &vec![
            ResourceResponse::Metas { metas: vec![] },
            ResourceResponse::MetasDetailed {
                metas_detailed: vec![],
            },
            ResourceResponse::Meta {
                meta: MetaItem::default(),
            },
            ResourceResponse::Streams { streams: vec![] },
            ResourceResponse::Subtitles { subtitles: vec![] },
            ResourceResponse::Addons { addons: vec![] },
        ],
        &[
            vec![
                Token::Seq { len: Some(6) },
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("metas"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("metasDetailed"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("meta"),
            ],
            MetaItem::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("streams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("subtitles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
}

#[test]
fn resource_response_streaming_de() {
    let stream = Stream {
        source: StreamSource::Url {
            url: "https://example.com/video.mp4".parse().unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints::default(),
    };

    let response = serde_json::from_str::<ResourceResponseCache>(
        r#"{"cacheMaxAge": 60, "unknown": {"nested": [1, {"key": null}]}, "streams": [{"url": "https://example.com/video.mp4"}, {"invalid": true}, 5]}"#,
    )
    .expect("Should deserialize");
    assert_eq!(response.cache_max_age, Some(60));
    assert_eq!(
        response.resource,
        ResourceResponse::Streams {
            streams: vec![stream.clone()]
        }
    );

    assert_eq!(
        serde_json::from_str::<ResourceResponse>(r#"{"subtitles": null}"#).unwrap(),
        ResourceResponse::Subtitles { subtitles: vec![] }
    );
    assert_eq!(
        serde_json::from_str::<ResourceResponse>(
            r#"{"streams": {}, "streams": [{"url": "https://example.com/video.mp4"}]}"#
        )
        .unwrap(),
        ResourceResponse::Streams {
            streams: vec![stream.clone()]
        }
    );

    for json in [
        r#"{"streams": 5, "metas": []}"#,
        r#"{"metas": [], "unknown": 1, "streams": []}"#,
    ] {
        assert!(serde_json::from_str::<ResourceResponse>(json)
            .expect_err("Should be an error")
            .to_string()
            .contains("Object response contained more than 1 of the unique keys"));
    }
    assert_eq!(
        serde_path_to_error::deserialize::<_, ResourceResponse>(
            &mut serde_json::Deserializer::from_str(r#"{"metas":{"object_key":"value"}}"#)
        )
        .expect_err("Should be an error")
        .to_string(),
        "invalid type: map, expected a sequence"
    );
    assert!(
        serde_json::from_str::<ResourceResponse>(r#"{"streams": "x"}"#)
            .expect_err("Should be an error")
            .to_string()
            .contains(r#"invalid type: string "x", expected a sequence"#)
    );
    assert!(serde_json::from_str::<ResourceResponse>("[]")
        .expect_err("Should be an error")
        .to_string()
        .contains("expected an Object response"));
    assert!(
        serde_json::from_str::<ResourceResponse>(r#"{"cacheMaxAge": 60}"#)
            .expect_err("Should be an error")
            .to_string()
            .contains("didn't contain any of the required keys")
    );

    let error = serde_json::from_str::<ResourceResponse>(
        r#"{"streams": [{"url": "https://example.com/video.mp4"}, {"name": "x", 5: 1}]}"#,
    )
    .expect_err("Should be an error");
    assert!(error
        .to_string()
        .starts_with("streams[1].?: key must be a string"));

    assert_de_tokens(
        &ResourceResponse::Streams {
            streams: vec![stream],
        }
        .readable(),
        &[
            Token::Map { len: None },
            Token::Str("unknown"),
            Token::Unit,
            Token::Str("streams"),
            Token::Seq { len: None },
            Token::Map { len: None },
            Token::Str("url"),
            Token::Str("https://example.com/video.mp4"),
            Token::MapEnd,
            Token::Str("invalid"),
            Token::SeqEnd,
            Token::MapEnd,
        ],
    );
}
