use crate::types::resource::Stream;
use crate::types::resource::{
    MetaItem, MetaItemPreview, SeriesInfo, StreamBehaviorHints, StreamSource, Video, VideoEpgInfo,
};
use crate::unit_tests::serde::default_tokens_ext::{DefaultFlattenTokens, DefaultTokens};
use chrono::{TimeZone, Utc};
use serde_test::{assert_de_tokens, assert_tokens, Configure, Token};

#[test]
fn video() {
    assert_tokens(
        &vec![
            Video {
                id: "id".into(),
                title: "title".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                overview: Some("overview".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                streams: vec![],
                series_info: Some(SeriesInfo::default()),
                epg_info: None,
                trailer_streams: vec![],
            },
            Video {
                id: "id".into(),
                title: "title".to_owned(),
                released: None,
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: None,
                epg_info: None,
                trailer_streams: vec![],
            },
        ]
        .readable(),
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("title"),
                Token::Str("title"),
                Token::Str("released"),
                Token::Some,
                Token::Str("2020-01-01T00:00:00Z"),
                Token::Str("overview"),
                Token::Some,
                Token::Str("overview"),
                Token::Str("thumbnail"),
                Token::Some,
                Token::Str("thumbnail"),
                Token::Str("streams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
            ],
            SeriesInfo::default_flatten_tokens(),
            vec![
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("title"),
                Token::Str("title"),
                Token::Str("released"),
                Token::None,
                Token::Str("overview"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("streams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
    assert_de_tokens(
        &vec![
            Video {
                id: "id".into(),
                title: "".to_owned(),
                released: None,
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: None,
                epg_info: None,
                trailer_streams: vec![],
            },
            Video {
                id: "id".into(),
                title: "title".to_owned(),
                released: None,
                overview: None,
                thumbnail: None,
                streams: vec![Stream {
                    source: StreamSource::default(),
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: StreamBehaviorHints::default(),
                }],
                series_info: None,
                epg_info: None,
                trailer_streams: vec![],
            },
            Video {
                id: "id".into(),
                title: "title".to_owned(),
                released: None,
                overview: None,
                thumbnail: None,
                streams: vec![Stream {
                    source: StreamSource::default(),
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: StreamBehaviorHints::default(),
                }],
                series_info: None,
                epg_info: None,
                trailer_streams: vec![],
            },
        ]
        .readable(),
        &[
            vec![
                Token::Seq { len: Some(3) },
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("id"),
                Token::MapEnd,
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("title"),
                Token::Str("title"),
                Token::Str("released"),
                Token::None,
                Token::Str("overview"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("stream"),
                Token::Map { len: None },
            ],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("name"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("subtitles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            StreamBehaviorHints::default_tokens(),
            vec![
                Token::MapEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("id"),
                Token::Str("title"),
                Token::Str("title"),
                Token::Str("released"),
                Token::None,
                Token::Str("overview"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("streams"),
                Token::Seq { len: Some(1) },
                Token::Map { len: None },
            ],
            StreamSource::default_flatten_tokens(),
            vec![
                Token::Str("name"),
                Token::None,
                Token::Str("description"),
                Token::None,
                Token::Str("thumbnail"),
                Token::None,
                Token::Str("subtitles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("behaviorHints"),
            ],
            StreamBehaviorHints::default_tokens(),
            vec![
                Token::MapEnd,
                Token::SeqEnd,
                Token::Str("trailerStreams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::MapEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
}

#[test]
fn videos_minimal() {
    assert_de_tokens(
        &MetaItem {
            preview: MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "".to_owned(),
                ..Default::default()
            },
            // Nothing to sort against. The ordering is from the addon
            videos: vec![
                Video {
                    id: "2".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "1".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "3".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
            ],
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItem",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("videos"),
            Token::Some,
            Token::Seq { len: None },
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("2"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("1"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("3"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn videos_released_equal() {
    assert_de_tokens(
        &MetaItem {
            preview: MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "".to_owned(),
                ..Default::default()
            },
            // All have same date. The ordering is from the addon
            videos: vec![
                Video {
                    id: "2".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "1".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "3".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
            ],
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItem",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("videos"),
            Token::Some,
            Token::Seq { len: None },
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("2"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("1"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("3"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn videos_released_sequal() {
    assert_de_tokens(
        &MetaItem {
            preview: MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "".to_owned(),
                ..Default::default()
            },
            // There is no series_info. Order by date descending.
            // If no date - at the end and the order is defined by addon
            videos: vec![
                Video {
                    id: "3".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 3, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "2".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "1".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "nd1".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "nd2".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
            ],
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItem",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("videos"),
            Token::Some,
            Token::Seq { len: None },
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("nd1"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("nd2"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("2"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-02-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("1"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("3"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-03-01T00:00:00Z"),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn various_videos_deserialization() {
    assert_de_tokens(
        &MetaItem {
            preview: MetaItemPreview {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: "".to_owned(),
                ..Default::default()
            },
            // Sort by season, then episode. Special at the end.
            // If no series_info sort by date ascending
            // If no date - sort to the end. Preserve order from addon
            videos: vec![
                Video {
                    id: "S01E01".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: Some(SeriesInfo {
                        season: 1,
                        episode: 1,
                    }),
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "S01E02".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: Some(SeriesInfo {
                        season: 1,
                        episode: 2,
                    }),
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "S02E01".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 3, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: Some(SeriesInfo {
                        season: 2,
                        episode: 1,
                    }),
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "special1".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 5, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: Some(SeriesInfo {
                        season: 0,
                        episode: 1,
                    }),
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "special2".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 5, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: Some(SeriesInfo {
                        season: 0,
                        episode: 2,
                    }),
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "M1".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "M2".to_owned(),
                    title: "".to_owned(),
                    released: Some(Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap()),
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "nd1".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
                Video {
                    id: "nd2".to_owned(),
                    title: "".to_owned(),
                    released: None,
                    overview: None,
                    thumbnail: None,
                    streams: vec![],
                    series_info: None,
                    epg_info: None,
                    trailer_streams: vec![],
                },
            ],
        }
        .readable(),
        &[
            Token::Struct {
                name: "MetaItem",
                len: 2,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("videos"),
            Token::Some,
            Token::Seq { len: None },
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("special2"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-05-01T00:00:00Z"),
            Token::Str("season"),
            Token::I32(0),
            Token::Str("episode"),
            Token::I32(2),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("S01E02"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-02-01T00:00:00Z"),
            Token::Str("season"),
            Token::I32(1),
            Token::Str("episode"),
            Token::I32(2),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("special1"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-05-01T00:00:00Z"),
            Token::Str("season"),
            Token::I32(0),
            Token::Str("episode"),
            Token::I32(1),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("S01E01"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::Str("season"),
            Token::I32(1),
            Token::Str("episode"),
            Token::I32(1),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("M2"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-02-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("M1"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-01-01T00:00:00Z"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("nd1"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("nd2"),
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("S02E01"),
            Token::Str("released"),
            Token::Some,
            Token::Str("2020-03-01T00:00:00Z"),
            Token::Str("season"),
            Token::I32(2),
            Token::Str("episode"),
            Token::I32(1),
            Token::MapEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn video_epg_info() {
    let video = serde_json::from_value::<Video>(serde_json::json!({
        "id": "pure:axn:2026-07-02T11:55",
        "title": "Spy x Family",
        "overview": "overview",
        "thumbnail": "thumbnail",
        "released": "2026-07-02T11:55:00Z",
        "startTime": "2026-07-02T11:55:00Z",
        "endTime": "2026-07-02T12:23:00Z",
        "runtime": "28 min",
        "releaseInfo": "2022",
        "genres": ["Action", "Anime"],
        "cast": ["Takuya Eguchi"],
        "directors": ["Kazuhiro Furuhashi"],
        "links": [],
    }))
    .expect("Failed to deserialize Video with EPG info");
    let epg_info = video.epg_info.as_ref().expect("epg_info should be present");
    assert_eq!(
        epg_info.start_time,
        Utc.with_ymd_and_hms(2026, 7, 2, 11, 55, 0).unwrap()
    );
    assert_eq!(epg_info.runtime.as_deref(), Some("28 min"));
    assert_eq!(epg_info.release_info.as_deref(), Some("2022"));
    assert_eq!(epg_info.genres, vec!["Action", "Anime"]);
    // on air only within [startTime, endTime)
    assert!(!epg_info.is_live(Utc.with_ymd_and_hms(2026, 7, 2, 11, 54, 59).unwrap()));
    assert!(epg_info.is_live(Utc.with_ymd_and_hms(2026, 7, 2, 11, 55, 0).unwrap()));
    assert!(epg_info.is_live(Utc.with_ymd_and_hms(2026, 7, 2, 12, 22, 59).unwrap()));
    assert!(!epg_info.is_live(Utc.with_ymd_and_hms(2026, 7, 2, 12, 23, 0).unwrap()));

    // videos without EPG fields must keep deserializing with no epg_info
    let video = serde_json::from_value::<Video>(serde_json::json!({
        "id": "tt123:1:1",
        "title": "regular video",
        "season": 1,
        "episode": 1,
    }))
    .expect("Failed to deserialize Video without EPG info");
    assert!(video.epg_info.is_none());
    assert_eq!(
        video.series_info,
        Some(SeriesInfo {
            season: 1,
            episode: 1
        })
    );

    // round-trip: epg fields serialize flattened in camelCase
    let epg_info = VideoEpgInfo {
        start_time: Utc.with_ymd_and_hms(2026, 7, 2, 11, 55, 0).unwrap(),
        end_time: Utc.with_ymd_and_hms(2026, 7, 2, 12, 23, 0).unwrap(),
        runtime: None,
        release_info: None,
        genres: vec![],
        cast: vec![],
        directors: vec![],
        links: vec![],
    };
    let json = serde_json::to_value(Video {
        id: "id".to_owned(),
        title: "title".to_owned(),
        released: None,
        overview: None,
        thumbnail: None,
        streams: vec![],
        series_info: None,
        epg_info: Some(epg_info.clone()),
        trailer_streams: vec![],
    })
    .unwrap();
    assert_eq!(json["startTime"], "2026-07-02T11:55:00Z");
    assert_eq!(json["endTime"], "2026-07-02T12:23:00Z");
    let roundtrip = serde_json::from_value::<Video>(json).unwrap();
    assert_eq!(roundtrip.epg_info, Some(epg_info));
}
