use crate::types::resource::{SeriesInfo, Video};
use chrono::prelude::TimeZone;
use chrono::Utc;

#[test]
fn deserialize_video() {
    let videos = vec![
        Video {
            id: "id1".to_owned(),
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
        },
        Video {
            id: "id2".to_owned(),
            title: "".to_owned(),
            released: None,
            overview: None,
            thumbnail: None,
            streams: vec![],
            series_info: None,
            trailer_streams: vec![],
        },
    ];
    let videos_json = r#"
    [
        {
            "id": "id1",
            "title": "title",
            "released": "2020-01-01T00:00:00Z",
            "overview": "overview",
            "thumbnail": "thumbnail",
            "streams": [],
            "season": 1,
            "episode": 1,
            "trailerStreams": []
        },
        {
            "id": "id2",
            "released": null,
            "overview": null,
            "thumbnail": null
        }
    ]
    "#;
    let videos_deserialize: Vec<Video> = serde_json::from_str(&videos_json).unwrap();
    assert_eq!(
        videos, videos_deserialize,
        "Video deserialized successfully"
    );
}
