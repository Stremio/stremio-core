use crate::types::addon::ResourceResponse;
use crate::types::addon::ResourceResponse::{Meta, Metas, MetasDetailed};
use crate::types::resource::{
    Link, MetaItem, MetaItemBehaviorHints, MetaItemPreview, SeriesInfo, Subtitles, Video,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use url::Url;

#[test]
fn deserialize_resource_response_metas() {
    let metas_vec = vec![MetaItemPreview {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        trailers: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: None,
            featured_video_id: None,
        },
    }];
    let metas_json = r#"
    {
        "metas": [
            {
                "id": "id",
                "type": "type_name",
                "name": "name",
                "poster": null,
                "logo": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null,
                "posterShape": "poster",
                "trailers": [],
                "behaviorHints": {
                    "defaultVideoId": null,
                    "featuredVideoId": null
                }
            }
        ]
    }
    "#;
    let metas_deserialize = serde_json::from_str(&metas_json).unwrap();
    match metas_deserialize {
        Metas { metas } => assert_eq!(metas, metas_vec, "metas deserialized successfully"),
        _ => panic!("failed getting metas"),
    };
}

#[test]
fn deserialize_resource_response_metas_detailed() {
    let metas_detailed_vec = vec![MetaItem {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        background: None,
        logo: None,
        popularity: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        videos: vec![Video {
            id: "id".to_owned(),
            title: "title".to_owned(),
            released: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            overview: None,
            thumbnail: None,
            streams: vec![],
            series_info: Some(SeriesInfo {
                season: 1,
                episode: 1,
            }),
            trailers: vec![],
        }],
        links: vec![Link {
            name: "name".to_owned(),
            category: "category".to_owned(),
            url: Url::parse("https://url").unwrap(),
        }],
        trailers: vec![],
        behavior_hints: Default::default(),
    }];
    let metas_detailed_json = r#"
    {
        "metasDetailed": [
            {
                "id": "id",
                "type": "type_name",
                "name": "name",
                "poster": null,
                "background": null,
                "logo": null,
                "popularity": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null,
                "posterShape": "poster",
                "videos": [
                    {
                        "id": "id",
                        "title": "title",
                        "released": "2020-01-01T00:00:00Z",
                        "overview": null,
                        "thumbnail": null,
                        "streams": [],
                        "season": 1,
                        "episode": 1,
                        "trailers": []
                    }
                ],
                "links": [
                    {
                        "name": "name",
                        "category": "category",
                        "url": "https://url/"
                    }
                ],
                "trailers": [],
                "behaviorHints": {
                    "defaultVideoId": null,
                    "featuredVideoId": null
                }
            }
        ]
    }
    "#;
    let metas_detailed_deserialize = serde_json::from_str(&metas_detailed_json).unwrap();
    match metas_detailed_deserialize {
        MetasDetailed { metas_detailed } => assert_eq!(
            metas_detailed, metas_detailed_vec,
            "metas detailed deserialized successfully"
        ),
        _ => panic!("failed getting metas detailed"),
    };
}

#[test]
fn deserialize_resource_response_meta() {
    let meta_item = MetaItem {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        background: None,
        logo: None,
        popularity: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        videos: vec![Video {
            id: "id".to_owned(),
            title: "title".to_owned(),
            released: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            overview: None,
            thumbnail: None,
            streams: vec![],
            series_info: Some(SeriesInfo {
                season: 1,
                episode: 1,
            }),
            trailers: vec![],
        }],
        links: vec![Link {
            name: "name".to_owned(),
            category: "category".to_owned(),
            url: Url::parse("https://url").unwrap(),
        }],
        trailers: vec![],
        behavior_hints: Default::default(),
    };
    let meta_json = r#"
    {
        "meta": {
            "id": "id",
            "type": "type_name",
            "name": "name",
            "poster": null,
            "background": null,
            "logo": null,
            "popularity": null,
            "description": null,
            "releaseInfo": null,
            "runtime": null,
            "released": null,
            "posterShape": "poster",
            "videos": [
                {
                    "id": "id",
                    "title": "title",
                    "released": "2020-01-01T00:00:00Z",
                    "overview": null,
                    "thumbnail": null,
                    "streams": [],
                    "season": 1,
                    "episode": 1,
                    "trailers": []
                }
            ],
            "links": [
                {
                    "name": "name",
                    "category": "category",
                    "url": "https://url/"
                }
            ],
            "trailers": [],
            "behaviorHints": {
                "defaultVideoId": null,
                "featuredVideoId": null
            }
        }
    }
    "#;
    let meta_deserialize = serde_json::from_str(&meta_json).unwrap();
    match meta_deserialize {
        Meta { meta } => assert_eq!(meta, meta_item, "meta deserialized successfully"),
        _ => panic!("failed getting meta"),
    };
}

#[test]
fn deserialize_resource_response_subtitles() {
    let subtitles_vec = vec![Subtitles {
        id: "id".to_owned(),
        lang: "lang".to_owned(),
        url: Url::parse("https://url").unwrap(),
    }];
    let subtitles_json = r#"
    {
        "subtitles": [
            {
                "id": "id",
                "lang": "lang",
                "url": "https://url/"
            }
        ]
    }
    "#;
    let subtitles_deserialize = serde_json::from_str(&subtitles_json).unwrap();
    match subtitles_deserialize {
        ResourceResponse::Subtitles { subtitles } => assert_eq!(
            subtitles, subtitles_vec,
            "subtitles deserialized successfully"
        ),
        _ => panic!("failed getting subtitles"),
    };
}
