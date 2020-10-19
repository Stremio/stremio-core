use crate::types::resource::{MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_meta_item_preview() {
    let meta_item_previews = vec![
        MetaItemPreview {
            id: "id1".to_owned(),
            r#type: "type".to_owned(),
            name: "name".to_owned(),
            poster: Some("poster".to_owned()),
            logo: Some("logo".to_owned()),
            description: Some("description".to_owned()),
            release_info: Some("release_info".to_owned()),
            runtime: Some("runtime".to_owned()),
            released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            poster_shape: PosterShape::Square,
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        },
        MetaItemPreview {
            id: "id2".to_owned(),
            r#type: "type".to_owned(),
            name: "".to_owned(),
            poster: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            released: None,
            poster_shape: PosterShape::Poster,
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        },
        MetaItemPreview {
            id: "id3".to_owned(),
            r#type: "type".to_owned(),
            name: "name".to_owned(),
            poster: None,
            logo: None,
            description: None,
            release_info: None,
            runtime: None,
            released: None,
            poster_shape: PosterShape::Poster,
            trailer_streams: vec![],
            behavior_hints: MetaItemBehaviorHints::default(),
        },
    ];
    let meta_item_previews_json = format!(
        r#"
        [
            {{
                "id": "id1",
                "type": "type",
                "name": "name",
                "poster": "poster",
                "logo": "logo",
                "description": "description",
                "releaseInfo": "release_info",
                "runtime": "runtime",
                "released": "2020-01-01T00:00:00Z",
                "posterShape": "square",
                "trailerStreams": [],
                "behaviorHints": {}
            }},
            {{
                "id": "id2",
                "type": "type",
                "poster": null,
                "logo": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null
            }},
            {{
                "id": "id3",
                "type": "type",
                "name": "name",
                "poster": null,
                "logo": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null,
                "posterShape": "invalid",
                "trailerStreams": [],
                "behaviorHints": {}
            }}
        ]
        "#,
        serde_json::to_string(&MetaItemBehaviorHints::default()).unwrap(),
        serde_json::to_string(&MetaItemBehaviorHints::default()).unwrap()
    );
    let meta_item_previews_deserialize: Vec<MetaItemPreview> =
        serde_json::from_str(&meta_item_previews_json).unwrap();
    assert_eq!(
        meta_item_previews, meta_item_previews_deserialize,
        "MetaItemPreview deserialized successfully"
    );
}