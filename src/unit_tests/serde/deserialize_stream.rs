use crate::types::resource::{Stream, StreamSource};
use url::Url;

#[test]
fn deserialize_stream() {
    let streams = vec![
        Stream {
            source: StreamSource::Url {
                url: Url::parse("https://url").unwrap(),
            },
            title: Some("title".to_owned()),
            thumbnail: Some("thumbnail".to_owned()),
            subtitles: vec![],
            behavior_hints: vec![("behavior_hint".to_owned(), serde_json::Value::Bool(true))]
                .iter()
                .cloned()
                .collect(),
        },
        Stream {
            source: StreamSource::Url {
                url: Url::parse("https://url").unwrap(),
            },
            title: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: vec![].iter().cloned().collect(),
        },
    ];
    let streams_json = r#"
    [
        {
            "url": "https://url",
            "title": "title",
            "thumbnail": "thumbnail",
            "behaviorHints": {
                "behavior_hint": true
            }
        },
        {
            "url": "https://url",
            "title": null,
            "thumbnail": null
        }
    ]
    "#;
    let streams_deserialize: Vec<Stream> = serde_json::from_str(&streams_json).unwrap();
    assert_eq!(
        streams, streams_deserialize,
        "Stream deserialized successfully"
    );
}
