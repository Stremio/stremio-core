use crate::types::resource::{Link, MetaItem, MetaItemBehaviorHints, PosterShape};
use chrono::prelude::TimeZone;
use chrono::Utc;
use url::Url;

#[test]
fn deserialize_meta_item() {
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
            videos: vec![],
            links: vec![Link {
                name: "name".to_owned(),
                category: "category".to_owned(),
                url: Url::parse("https://url").unwrap(),
            }],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
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
            behavior_hints: MetaItemBehaviorHints::default(),
        },
        // ALL NONEs are set to null.
        // poster shape is invalid
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
            videos: vec![],
            links: vec![],
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
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
            "videos": [],
            "links": [
                {
                    "name": "name",
                    "category": "category",
                    "url": "https://url"
                }
            ],
            "trailerStreams": [],
            "behaviorHints": {
                "defaultVideoId": null,
                "featuredVideoId": null,
                "has_scheduled_videos": false
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
            "videos": [],
            "links": [],
            "trailerStreams": [],
            "behaviorHints": {
                "defaultVideoId": null,
                "featuredVideoId": null,
                "has_scheduled_videos": false
            }
        }
    ]
    "#;
    let meta_items_deserialize: Vec<MetaItem> = serde_json::from_str(&meta_items_json).unwrap();
    assert_eq!(
        meta_items, meta_items_deserialize,
        "MetaItem deserialized successfully"
    );
}
