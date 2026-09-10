use crate::types::addon::ResourceResponse;
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
            fonts: vec![],
            other: Default::default(),
        },
        &[
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("lang"),
            Token::Str("lang"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::MapEnd,
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
            fonts: vec![],
            other: Default::default(),
        },
        &[
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("lang"),
            Token::Str("eng"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::Str("label"),
            Token::Some,
            Token::Str("eng #1 [opensubtitles] 1080p.BluRay"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn subtitles_with_fonts() {
    assert_tokens(
        &Subtitles {
            id: "id".into(),
            lang: "eng".to_owned(),
            url: Url::parse("https://url").unwrap(),
            label: None,
            fonts: vec![
                Url::parse("https://example.com/font.ttf").unwrap(),
                Url::parse("https://example.com/font.otf").unwrap(),
            ],
            other: Default::default(),
        },
        &[
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("lang"),
            Token::Str("eng"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::Str("fonts"),
            Token::Seq { len: Some(2) },
            Token::Str("https://example.com/font.ttf"),
            Token::Str("https://example.com/font.otf"),
            Token::SeqEnd,
            Token::MapEnd,
        ],
    );
}

/// The add-on specific properties, e.g. the ones the OpenSubtitles v3 add-on
/// sends, must survive deserialization of a subtitles resource response.
#[test]
fn subtitles_response_keeps_add_on_specific_properties() {
    let response = serde_json::json!({
        "subtitles": [
            {
                "id": "1",
                "url": "https://opensubtitles.example/1.srt",
                "lang": "eng",
                "SubEncoding": "CP1252",
                "subtitleFileName": "Movie.2009.1080p.BluRay.x264-GROUP.srt",
                "movieReleaseName": "Movie.2009.1080p.BluRay.x264-GROUP",
                "releaseGroup": "GROUP",
                "fpsMilli": 23976
            }
        ]
    });

    let subtitles = match serde_json::from_value::<ResourceResponse>(response).unwrap() {
        ResourceResponse::Subtitles { subtitles } => subtitles,
        _ => panic!("Whoops, wrong variant!"),
    };
    let subtitle = &subtitles[0];

    assert_eq!("1", subtitle.id);
    assert_eq!("eng", subtitle.lang);
    assert_eq!(
        Url::parse("https://opensubtitles.example/1.srt").unwrap(),
        subtitle.url
    );
    assert_eq!(None, subtitle.label);
    assert!(subtitle.fonts.is_empty());
    assert_eq!(
        Some(&serde_json::json!("Movie.2009.1080p.BluRay.x264-GROUP.srt")),
        subtitle.other.get("subtitleFileName")
    );
    assert_eq!(
        Some(&serde_json::json!("Movie.2009.1080p.BluRay.x264-GROUP")),
        subtitle.other.get("movieReleaseName")
    );
    assert_eq!(
        Some(&serde_json::json!("GROUP")),
        subtitle.other.get("releaseGroup")
    );
    assert_eq!(
        Some(&serde_json::json!(23976)),
        subtitle.other.get("fpsMilli")
    );
    assert_eq!(
        Some(&serde_json::json!("CP1252")),
        subtitle.other.get("SubEncoding")
    );
    assert_eq!(5, subtitle.other.len());
}

/// A subtitle without any add-on specific properties deserializes exactly as
/// it did before and serializes without any additional keys.
#[test]
fn subtitles_without_add_on_specific_properties() {
    let subtitle = serde_json::from_value::<Subtitles>(serde_json::json!({
        "id": "1",
        "url": "https://opensubtitles.example/1.srt",
        "lang": "eng"
    }))
    .unwrap();

    assert!(subtitle.other.is_empty());
    assert_eq!(
        serde_json::json!({
            "id": "1",
            "url": "https://opensubtitles.example/1.srt",
            "lang": "eng"
        }),
        serde_json::to_value(&subtitle).unwrap()
    );
}

/// The add-on specific properties are serialized back out, so that a
/// round-trip through the core does not lose them.
#[test]
fn subtitles_add_on_specific_properties_round_trip() {
    let json = serde_json::json!({
        "id": "1",
        "url": "https://opensubtitles.example/1.srt",
        "lang": "eng",
        "label": "English (GROUP)",
        "subtitleFileName": "Movie.2009.1080p.BluRay.x264-GROUP.srt",
        "releaseGroup": "GROUP",
        "fpsMilli": 23976
    });

    let subtitle = serde_json::from_value::<Subtitles>(json.clone()).unwrap();
    assert_eq!(json, serde_json::to_value(&subtitle).unwrap());
    assert_eq!(
        subtitle,
        serde_json::from_value::<Subtitles>(serde_json::to_value(&subtitle).unwrap()).unwrap()
    );
}
