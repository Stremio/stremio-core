use crate::types::resource::{
    Link, MetaItemBehaviorHints, MetaItemPreview, PosterShape, Stream, StreamBehaviorHints,
    StreamSource,
};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, assert_ser_tokens, Configure, Token};
use url::Url;

#[test]
fn meta_item_preview_serialize() {
    assert_ser_tokens(
        &vec![
            MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "name".to_owned(),
                poster: Some(Url::parse("http://poster/").unwrap()),
                background: Some(Url::parse("http://background/").unwrap()),
                logo: Some(Url::parse("http://logo/").unwrap()),
                description: Some("description".to_owned()),
                release_info: Some("release_info".to_owned()),
                runtime: Some("runtime".to_owned()),
                age_rating: None,
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                poster_shape: PosterShape::default(),
                links: vec![],
                trailer_streams: vec![],
                behavior_hints: MetaItemBehaviorHints::default(),
            },
            MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "name".to_owned(),
                poster: None,
                background: None,
                logo: None,
                description: None,
                release_info: None,
                runtime: None,
                age_rating: None,
                released: None,
                poster_shape: PosterShape::default(),
                links: vec![],
                trailer_streams: vec![],
                behavior_hints: MetaItemBehaviorHints::default(),
            },
        ]
        .readable(),
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "MetaItemPreview",
                    len: 14,
                },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("type"),
                Token::Str("type"),
                Token::Str("name"),
                Token::Str("name"),
                Token::Str("poster"),
                Token::Some,
                Token::Str("http://poster/"),
                Token::Str("background"),
                Token::Some,
                Token::Str("http://background/"),
                Token::Str("logo"),
                Token::Some,
                Token::Str("http://logo/"),
                Token::Str("description"),
                Token::Some,
                Token::Str("description"),
                Token::Str("releaseInfo"),
                Token::Some,
                Token::Str("release_info"),
                Token::Str("runtime"),
                Token::Some,
                Token::Str("runtime"),
                Token::Str("released"),
                Token::Some,
                Token::Str("2020-01-01T00:00:00Z"),
                Token::Str("posterShape"),
            ],
            PosterShape::default_tokens(),
            vec![
                Token::Str("links"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            MetaItemBehaviorHints::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "MetaItemPreview",
                    len: 14,
                },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("type"),
                Token::Str("type"),
                Token::Str("name"),
                Token::Str("name"),
                Token::Str("poster"),
                Token::None,
                Token::Str("background"),
                Token::None,
                Token::Str("logo"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("releaseInfo"),
                Token::None,
                Token::Str("runtime"),
                Token::None,
                Token::Str("released"),
                Token::None,
                Token::Str("posterShape"),
            ],
            PosterShape::default_tokens(),
            vec![
                Token::Str("links"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            MetaItemBehaviorHints::default_tokens(),
            vec![Token::StructEnd, Token::SeqEnd],
        ]
        .concat(),
    );
}

#[test]
fn meta_item_preview_de() {
    assert_de_tokens(
        &vec![
            MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "name".to_owned(),
                poster: Some(Url::parse("http://poster/").unwrap()),
                background: Some(Url::parse("http://background/").unwrap()),
                logo: Some(Url::parse("http://logo/").unwrap()),
                description: Some("description".to_owned()),
                release_info: Some("release_info".to_owned()),
                runtime: Some("runtime".to_owned()),
                age_rating: None,
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                poster_shape: PosterShape::default(),
                links: vec![],
                trailer_streams: vec![],
                behavior_hints: MetaItemBehaviorHints::default(),
            },
            MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "name".to_owned(),
                poster: None,
                background: None,
                logo: None,
                description: None,
                release_info: None,
                runtime: None,
                age_rating: None,
                released: None,
                poster_shape: PosterShape::default(),
                links: vec![],
                trailer_streams: vec![],
                behavior_hints: MetaItemBehaviorHints::default(),
            },
        ]
        .readable(),
        &[
            vec![
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "MetaItemPreviewLegacy",
                    len: 14,
                },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("type"),
                Token::Str("type"),
                Token::Str("name"),
                Token::Some,
                Token::Str("name"),
                Token::Str("poster"),
                Token::Some,
                Token::Str("http://poster/"),
                Token::Str("background"),
                Token::Some,
                Token::Str("http://background/"),
                Token::Str("logo"),
                Token::Some,
                Token::Str("http://logo/"),
                Token::Str("description"),
                Token::Some,
                Token::Str("description"),
                Token::Str("releaseInfo"),
                Token::Some,
                Token::Str("release_info"),
                Token::Str("runtime"),
                Token::Some,
                Token::Str("runtime"),
                Token::Str("released"),
                Token::Some,
                Token::Str("2020-01-01T00:00:00Z"),
                Token::Str("posterShape"),
                Token::None,
                Token::Str("links"),
                Token::None,
                Token::Str("trailerStreams"),
                Token::None,
                Token::Str("behaviorHints"),
                Token::None,
            ],
            vec![Token::StructEnd],
            vec![
                Token::Struct {
                    name: "MetaItemPreviewLegacy",
                    len: 14,
                },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("type"),
                Token::Str("type"),
                Token::Str("name"),
                Token::Some,
                Token::Str("name"),
                Token::Str("poster"),
                Token::None,
                Token::Str("background"),
                Token::None,
                Token::Str("logo"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("releaseInfo"),
                Token::None,
                Token::Str("runtime"),
                Token::None,
                Token::Str("released"),
                Token::None,
                Token::Str("posterShape"),
                Token::None,
                Token::Str("links"),
                Token::None,
                Token::Str("trailerStreams"),
                Token::None,
                Token::Str("behaviorHints"),
                Token::None,
                Token::StructEnd,
            ],
            vec![Token::SeqEnd],
        ]
        .concat(),
    );
}

#[test]
fn meta_item_preview_de_minimal() {
    assert_de_tokens(
        &MetaItemPreview {
            id: "id".into(),
            r#type: "type".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            age_rating: None,
            released: None,
            poster_shape: PosterShape::default(),
            links: vec![],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItemPreviewLegacy",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn meta_item_preview_de_null() {
    assert_de_tokens(
        &MetaItemPreview {
            id: "tt:123456".to_owned(),
            r#type: "movie".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            age_rating: None,
            released: None,
            poster_shape: PosterShape::default(),
            links: vec![],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItemPreviewLegacy",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("tt:123456"),
            Token::Str("type"),
            Token::Str("movie"),
            Token::Str("name"),
            Token::None,
            Token::Str("poster"),
            Token::None,
            Token::Str("background"),
            Token::None,
            Token::Str("logo"),
            Token::None,
            Token::Str("description"),
            Token::None,
            Token::Str("released"),
            Token::None,
            Token::Str("posterShape"),
            Token::None,
            Token::Str("links"),
            Token::None,
            Token::Str("trailerStreams"),
            Token::None,
            Token::Str("behaviorHints"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn meta_item_preview_de_numeric_imdb() {
    assert_de_tokens(
        &MetaItemPreview {
            id: "id".into(),
            r#type: "type".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            age_rating: None,
            released: None,
            poster_shape: PosterShape::default(),
            links: vec![Link {
                name: "5.1".to_owned(),
                category: "imdb".to_owned(),
                url: Url::parse("https://imdb.com/title/id").unwrap(),
            }],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItemPreviewLegacy",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("imdbRating"),
            Token::Some,
            Token::F64(5.1),
            Token::StructEnd,
        ],
    );
}

#[test]
fn meta_item_preview_de_legacy_links() {
    assert_de_tokens(
        &MetaItemPreview {
            id: "id".into(),
            r#type: "type".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            age_rating: None,
            released: None,
            poster_shape: PosterShape::default(),
            links: vec![Link {
                name: "5.1".to_owned(),
                category: "imdb".to_owned(),
                url: Url::parse("https://imdb.com/title/id").unwrap(),
            },
            Link {
                name: "Action".to_owned(),
                category: "Genres".to_owned(),
                url: Url::parse("stremio:///discover/https%3A%2F%2Fv3-cinemeta.strem.io%2Fmanifest.json/type/top?genre=Action").unwrap(),
            }
            ],
            trailer_streams: vec![
                Stream {
                    source: StreamSource::YouTube {
                        yt_id: "source1".to_owned(),
                    },
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: StreamBehaviorHints::default(),
                },
                Stream {
                    source: StreamSource::YouTube {
                        yt_id: "source3".to_owned(),
                    },
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: StreamBehaviorHints::default(),
                }
            ],
            behavior_hints: MetaItemBehaviorHints::default(),
        }.readable(),
        &[
            Token::Struct {
                name: "MetaItemPreviewLegacy",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("imdbRating"),
            Token::Some,
            Token::Str("5.1"),
            Token::Str("genres"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Str("Action"),
            Token::SeqEnd,
            Token::Str("trailers"),
            Token::Some,
            Token::Seq { len: Some(3) },
            Token::Map{ len: None },
            Token::Str("source"),
            Token::Str("source1"),
            Token::Str("type"),
            Token::Str("Trailer"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("source"),
            Token::Str("source2"),
            Token::Str("type"),
            Token::Str("NotTrailer"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("ytId"), 
            Token::Str("source3"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn meta_item_preview_de_ignore_legacy_when_links() {
    assert_de_tokens(
        &MetaItemPreview {
            id: "id".into(),
            r#type: "type".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            age_rating: None,
            released: None,
            poster_shape: PosterShape::default(),
            links: vec![Link {
                name: "5.1".to_owned(),
                category: "imdb".to_owned(),
                url: Url::parse("https://imdb.com/title/id").unwrap(),
            },
            Link {
                name: "Action".to_owned(),
                category: "Genres".to_owned(),
                url: Url::parse("stremio:///discover/https%3A%2F%2Fv3-cinemeta.strem.io%2Fmanifest.json/type/top?genre=Action").unwrap(),
            }],
            trailer_streams: vec![Stream {
                source: StreamSource::YouTube {
                    yt_id: "trailer_stream".to_owned(),
                },
                name: None,
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: StreamBehaviorHints::default(),
            }],
            behavior_hints: MetaItemBehaviorHints::default(),
        }.readable(),
        &[
            Token::Struct {
                name: "MetaItemPreviewLegacy",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("imdbRating"),
            Token::Some,
            Token::Str("10"),
            Token::Str("genres"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Str("Drama"),
            Token::SeqEnd,
            Token::Str("trailers"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Map{ len: None },
            Token::Str("source"),
            Token::Str("source1"),
            Token::Str("type"),
            Token::Str("Trailer"),
            Token::MapEnd,
            Token::Map{ len: None },
            Token::Str("source"),
            Token::Str("source2"),
            Token::Str("type"),
            Token::Str("NotTrailer"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::Str("links"),
            Token::Some,
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "Link",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("5.1"),
            Token::Str("category"),
            Token::Str("imdb"),
            Token::Str("url"),
            Token::Str("https://imdb.com/title/id"),
            Token::StructEnd,
            Token::Struct {
                name: "Link",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("Action"),
            Token::Str("category"),
            Token::Str("Genres"),
            Token::Str("url"),
            Token::Str("stremio:///discover/https%3A%2F%2Fv3-cinemeta.strem.io%2Fmanifest.json/type/top?genre=Action"),
            Token::StructEnd,
            Token::SeqEnd,
            Token::Str("trailerStreams"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Map { len: None },
            Token::Str("ytId"), 
            Token::Str("trailer_stream"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn age_rating_survives_addon_catalog_and_meta_responses() {
    use crate::types::addon::ResourceResponse;
    use serde_json::{json, Value};

    let movie = json!({ "id": "tt1234567", "name": "Example movie", "type": "movie", "ageRating": "PG-13" });
    let series =
        json!({ "id": "tt1234568", "name": "Example series", "type": "series", "ageRating": 16 });

    let catalog: ResourceResponse = serde_json::from_value(json!({
        "metas": [
            movie,
            series,
            { "id": "tt1234569", "name": "Unrated movie", "type": "movie" },
            { "id": "tt1234570", "name": "Legacy addon movie", "type": "movie", "ageRating": { "label": "PG" } }
        ]
    }))
    .unwrap();
    let serialized = serde_json::to_value(catalog).unwrap();
    let ratings: Vec<_> = serialized["metas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|meta| meta.get("ageRating").and_then(Value::as_str))
        .collect();
    assert_eq!(ratings, [Some("PG-13"), Some("16"), None, None]);
    assert!(serialized["metas"][2].get("ageRating").is_none());
    assert!(serialized["metas"][3].get("ageRating").is_none());

    for meta in [movie, series] {
        let rating = if meta["type"] == "movie" {
            "PG-13"
        } else {
            "16"
        };
        let details: ResourceResponse = serde_json::from_value(json!({ "meta": meta })).unwrap();
        assert_eq!(
            serde_json::to_value(details).unwrap()["meta"]["ageRating"],
            rating
        );
    }
}
