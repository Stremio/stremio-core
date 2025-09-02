#[test]
fn meta_item_video_sorting_edge_cases() {
    use serde_json::json;
    // Mixed seriesInfo and missing fields
    let meta_json = json!({
        "id": "series2",
        "type": "series",
        "name": "Edge Series",
        "videos": [
            {"id": "noinfo"},
            {"id": "ep1", "seriesInfo": {"season": 1, "episode": 1}},
            {"id": "ep2", "seriesInfo": {"season": 1, "episode": 2}},
            {"id": "zero", "seriesInfo": {"season": 0, "episode": 0}},
            {"id": "dup", "seriesInfo": {"season": 1, "episode": 2}}
        ]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    let ids: Vec<_> = meta.videos.iter().map(|v| v.id.as_str()).collect();
    // Should sort by season/episode, 0s last, duplicates preserved
    assert_eq!(ids, ["ep1", "ep2", "dup", "zero", "noinfo"]);

    // Movie with release dates
    let meta_json = json!({
        "id": "movie2",
        "type": "movie",
        "name": "Edge Movie",
        "videos": [
            {"id": "a", "released": "2020-01-01T00:00:00Z"},
            {"id": "b", "released": "2021-01-01T00:00:00Z"},
            {"id": "c"}
        ]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    let ids: Vec<_> = meta.videos.iter().map(|v| v.id.as_str()).collect();
    // For movies, fallback is reverse lexicographical if no released, but with released, sort by released descending
    assert_eq!(ids, ["b", "a", "c"]);

    // Non-standard type
    let meta_json = json!({
        "id": "other1",
        "type": "documentary",
        "name": "Other",
        "videos": [
            {"id": "x"}, {"id": "y"}, {"id": "z"}
        ]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    let ids: Vec<_> = meta.videos.iter().map(|v| v.id.as_str()).collect();
    // Fallback is reverse lexicographical
    assert_eq!(ids, ["z", "y", "x"]);

    // Empty and single-element
    let meta_json = json!({
        "id": "empty",
        "type": "series",
        "name": "Empty",
        "videos": []
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    assert!(meta.videos.is_empty());
    let meta_json = json!({
        "id": "single",
        "type": "movie",
        "name": "Single",
        "videos": [{"id": "only"}]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    assert_eq!(meta.videos[0].id, "only");
}
use crate::types::resource::Video;

#[test]
fn meta_item_video_sorting_by_type() {
    use serde_json::json;
    // For series, should sort by season/episode ascending
    let meta_json = json!({
        "id": "series1",
        "type": "series",
        "name": "Test Series",
        "videos": [
            {"id": "ep2", "seriesInfo": {"season": 1, "episode": 2}},
            {"id": "ep1", "seriesInfo": {"season": 1, "episode": 1}},
            {"id": "ep3", "seriesInfo": {"season": 2, "episode": 1}}
        ]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    let ids: Vec<_> = meta.videos.iter().map(|v| v.id.as_str()).collect();
    assert_eq!(ids, ["ep1", "ep2", "ep3"]);

    // For movie, should sort by id descending (fallback logic)
    let meta_json = json!({
        "id": "movie1",
        "type": "movie",
        "name": "Test Movie",
        "videos": [
            {"id": "b"},
            {"id": "a"},
            {"id": "c"}
        ]
    });
    let meta: MetaItem = serde_json::from_value(meta_json).unwrap();
    let ids: Vec<_> = meta.videos.iter().map(|v| v.id.as_str()).collect();
    // For movies, fallback is reverse lexicographical (see cmp logic)
    assert_eq!(ids, ["c", "b", "a"]);
}
use crate::types::resource::{MetaItem, MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, assert_tokens, Token};
use url::Url;

#[test]
fn meta_item() {
    assert_tokens(
        &vec![
            MetaItem {
                preview: MetaItemPreview {
                    id: "tt:123456".to_owned(),
                    r#type: "movie".to_owned(),
                    name: "name".to_owned(),
                    poster: Some(Url::parse("http://poster/").unwrap()),
                    background: Some(Url::parse("http://background/").unwrap()),
                    logo: Some(Url::parse("http://logo/").unwrap()),
                    description: Some("description".to_owned()),
                    release_info: Some("release_info".to_owned()),
                    runtime: Some("runtime".to_owned()),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    poster_shape: PosterShape::default(),
                    links: vec![],
                    trailer_streams: vec![],
                    behavior_hints: MetaItemBehaviorHints::default(),
                },
                videos: vec![],
            },
            MetaItem {
                preview: MetaItemPreview {
                    id: "tt:654321".to_owned(),
                    r#type: "movie".to_owned(),
                    name: "name".to_owned(),
                    poster: None,
                    background: None,
                    logo: None,
                    description: None,
                    release_info: None,
                    runtime: None,
                    released: None,
                    poster_shape: PosterShape::default(),
                    links: vec![],
                    trailer_streams: vec![],
                    behavior_hints: MetaItemBehaviorHints::default(),
                },
                videos: vec![],
            },
        ],
        &[
            vec![Token::Seq { len: Some(2) }],
            vec![
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("tt:123456"),
                Token::Str("type"),
                Token::Str("movie"),
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
                // Token::None,
                // Token::Some,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                // Token::None,
                // Token::Some,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
                // Token::None,
            ],
            MetaItemBehaviorHints::default_tokens(),
            vec![
                Token::Str("videos"),
                // Token::None,
                Token::Some,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
            ],
            vec![
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("tt:654321"),
                Token::Str("type"),
                Token::Str("movie"),
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
            vec![
                Token::Str("videos"),
                Token::Some,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
            ],
            vec![Token::SeqEnd],
        ]
        .concat(),
    );
    assert_de_tokens(
        &[
            MetaItem {
                preview: MetaItemPreview {
                    id: "id".into(),
                    r#type: "type".to_owned(),
                    name: "".to_owned(),
                    poster: None,
                    background: None,
                    logo: None,
                    description: None,
                    release_info: None,
                    runtime: None,
                    released: Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 10).unwrap()),
                    poster_shape: PosterShape::default(),
                    links: vec![],
                    trailer_streams: vec![],
                    behavior_hints: MetaItemBehaviorHints::default(),
                },
                videos: vec![],
            },
            MetaItem {
                preview: MetaItemPreview {
                    id: "id".into(),
                    r#type: "type".to_owned(),
                    name: "".to_owned(),
                    poster: None,
                    background: None,
                    logo: None,
                    description: None,
                    release_info: Some("1".to_owned()),
                    runtime: Some("2".to_owned()),
                    released: None,
                    poster_shape: PosterShape::default(),
                    links: vec![],
                    trailer_streams: vec![],
                    behavior_hints: MetaItemBehaviorHints::default(),
                },
                videos: vec![],
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "MetaItem",
                len: 3,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("released"),
            Token::Some,
            Token::I64(10_000),
            // videos field
            Token::Str("videos"),
            Token::None,
            Token::StructEnd,
            Token::Struct {
                name: "MetaItem",
                len: 4,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("releaseInfo"),
            Token::I32(1),
            Token::Str("runtime"),
            Token::I32(2),
            // videos field
            Token::Str("videos"),
            Token::None,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
}

#[test]
fn meta_item_de_urls_none_when_empty() {
    assert_de_tokens(
        &MetaItem {
            preview: MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "".to_owned(),
                poster: None,
                background: None,
                logo: None,
                description: None,
                release_info: None,
                runtime: None,
                released: None,
                poster_shape: PosterShape::default(),
                links: vec![],
                trailer_streams: vec![],
                behavior_hints: MetaItemBehaviorHints::default(),
            },
            videos: vec![],
        },
        &[
            Token::Struct {
                name: "MetaItem",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("poster"),
            Token::Some,
            Token::Str(""),
            Token::Str("background"),
            Token::Some,
            Token::Str(""),
            Token::Str("logo"),
            Token::Some,
            Token::Str(""),
            Token::Str("videos"),
            Token::None,
            Token::StructEnd,
        ],
    );
}
