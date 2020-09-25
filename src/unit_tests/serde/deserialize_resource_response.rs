use crate::types::addon::ResourceResponse;
use crate::types::resource::{
    Link, MetaItem, MetaItemBehaviorHints, PosterShape, SeriesInfo, Video,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use url::Url;

#[test]
fn deserialize_resource_response_metas() {
    let metas = ResourceResponse::Metas { metas: vec![] };
    let metas_json = r#"
    {
        "metas": []
    }
    "#;
    let metas_deserialize = serde_json::from_str(&metas_json).unwrap();
    assert_eq!(metas, metas_deserialize, "Metas deserialized successfully");
}

#[test]
fn deserialize_resource_response_metas_detailed() {
    let metas_detailed = ResourceResponse::MetasDetailed {
        metas_detailed: vec![],
    };
    let metas_detailed_json = r#"
    {
        "metasDetailed": []
    }
    "#;
    let metas_detailed_deserialize = serde_json::from_str(&metas_detailed_json).unwrap();
    assert_eq!(
        metas_detailed, metas_detailed_deserialize,
        "MetasDetailed deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_meta() {
    let meta_items = vec![
        // ALL fields are defined with SOME value
        MetaItem {
            id: "id1".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: Some("poster".to_owned()),
            background: Some("background".to_owned()),
            logo: Some("logo".to_owned()),
            popularity: Some(1.0),
            description: Some("description".to_owned()),
            release_info: Some("release_info".to_owned()),
            runtime: Some("runtime".to_owned()),
            released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            poster_shape: PosterShape::Square,
            videos: vec![Video {
                id: "id".to_owned(),
                title: "title".to_owned(),
                released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                overview: Some("overview".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 1,
                }),
                trailer_streams: vec![],
            }],
            links: vec![Link {
                name: "name".to_owned(),
                category: "category".to_owned(),
                url: Url::parse("https://url").unwrap(),
            }],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints {
                default_video_id: Some("default_video_id".to_owned()),
                featured_video_id: Some("featured_video_id".to_owned()),
                has_scheduled_videos: true,
            },
        },
        // serde(default) are omited
        MetaItem {
            id: "id2".to_owned(),
            type_name: "type_name".to_owned(),
            name: "".to_owned(),
            poster: None,
            background: None,
            logo: None,
            popularity: None,
            description: None,
            release_info: None,
            runtime: None,
            released: None,
            poster_shape: PosterShape::Poster,
            videos: vec![],
            links: vec![],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints {
                default_video_id: None,
                featured_video_id: None,
                has_scheduled_videos: false,
            },
        },
        // ALL NONEs are set to null.
        // poster shape is invalid
        // title, streams, trailer_streams, has_scheduled_videos are omited
        MetaItem {
            id: "id3".to_owned(),
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
            poster_shape: PosterShape::Poster,
            videos: vec![Video {
                id: "id".to_owned(),
                title: "".to_owned(),
                released: None,
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 1,
                }),
                trailer_streams: vec![],
            }],
            links: vec![],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints {
                default_video_id: None,
                featured_video_id: None,
                has_scheduled_videos: false,
            },
        },
    ];
    let meta_items_json = r#"
    [
        {
            "id": "id1",
            "type": "type_name",
            "name": "name",
            "poster": "poster",
            "background": "background",
            "logo": "logo",
            "popularity": 1.0,
            "description": "description",
            "releaseInfo": "release_info",
            "runtime": "runtime",
            "released": "2020-01-01T00:00:00Z",
            "posterShape": "square",
            "videos": [
                {
                    "id": "id",
                    "title": "title",
                    "released": "2020-01-01T00:00:00Z",
                    "overview": "overview",
                    "thumbnail": "thumbnail",
                    "streams": [],
                    "season": 1,
                    "episode": 1,
                    "trailerStreams": []
                }
            ],
            "links": [
                {
                    "name": "name",
                    "category": "category",
                    "url": "https://url"
                }
            ],
            "trailerStreams": [],
            "behaviorHints": {
                "defaultVideoId": "default_video_id",
                "featuredVideoId": "featured_video_id",
                "hasScheduledVideos": true
            }
        },
        {
            "id": "id2",
            "type": "type_name",
            "poster": null,
            "background": null,
            "logo": null,
            "popularity": null,
            "description": null,
            "releaseInfo": null,
            "runtime": null,
            "released": null
        },
        {
            "id": "id3",
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
            "posterShape": "invalid",
            "videos": [
                {
                    "id": "id",
                    "released": null,
                    "overview": null,
                    "thumbnail": null,
                    "season": 1,
                    "episode": 1
                }
            ],
            "links": [],
            "trailerStreams": [],
            "behaviorHints": {
                "defaultVideoId": null,
                "featuredVideoId": null
            }
        }
    ]
    "#;
    let meta_items_deserialize: Vec<MetaItem> = serde_json::from_str(&meta_items_json).unwrap();
    assert_eq!(
        meta_items, meta_items_deserialize,
        "meta items deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_streams() {
    let streams = ResourceResponse::Streams { streams: vec![] };
    let streams_json = r#"
    {
        "streams": []
    }
    "#;
    let streams_deserialize = serde_json::from_str(&streams_json).unwrap();
    assert_eq!(
        streams, streams_deserialize,
        "Streams deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_subtitles() {
    let subtitles = ResourceResponse::Subtitles { subtitles: vec![] };
    let subtitles_json = r#"
    {
        "subtitles": []
    }
    "#;
    let subtitles_deserialize = serde_json::from_str(&subtitles_json).unwrap();
    assert_eq!(
        subtitles, subtitles_deserialize,
        "Subtitles deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_addons() {
    let addons = ResourceResponse::Addons { addons: vec![] };
    let addons_json = r#"
    {
        "addons": []
    }
    "#;
    let addons_deserialize = serde_json::from_str(&addons_json).unwrap();
    assert_eq!(
        addons, addons_deserialize,
        "Addons deserialized successfully"
    );
}
