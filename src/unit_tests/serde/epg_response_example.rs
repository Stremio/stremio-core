use chrono::{TimeZone, Utc};

use crate::types::addon::ResourceResponse;

#[test]
fn epg_response_example() {
    let json = serde_json::json!({
        "metasDetailed": [
            {
                "id": "pure:axn",
                "type": "tv",
                "name": "AXN",
                "logo": "https://addon.example.com/logos/axn.png",
                "poster": "https://addon.example.com/logos/axn.png",
                "posterShape": "landscape",
                "behaviorHints": { "hasScheduledVideos": true },
                "videos": [
                    {
                        "id": "pure:axn:8f3k2j",
                        "title": "S.W.A.T.",
                        "overview": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Hondo e a equipe enfrentam um dilema urbano.",
                        "thumbnail": "https://addon.example.com/thumbs/swat.jpg",
                        "released": "2026-07-02T10:30:00.000Z",
                        "startTime": "2026-07-02T10:30:00.000Z",
                        "endTime": "2026-07-02T11:55:00.000Z",
                        "runtime": "85 min",
                        "releaseInfo": "2018",
                        "genres": ["Ação", "Drama", "Policial"],
                        "cast": ["Shemar Moore", "Alex Russell"],
                        "directors": ["Justin Lin"],
                        "links": [],
                        "streams": [
                            {
                                "name": "PureTV",
                                "description": "AXN",
                                "url": "https://cdn.example.com/axn/master.m3u8",
                                "behaviorHints": { "notWebReady": true }
                            }
                        ]
                    },
                    {
                        "id": "pure:axn:1m9x4p",
                        "title": "Spy x Family",
                        "overview": "Ut enim ad minim veniam, quis nostrud exercitation. Loid, Yor e Anya em mais uma missão em família.",
                        "thumbnail": "https://addon.example.com/thumbs/spy-x-family.jpg",
                        "released": "2026-07-02T11:55:00.000Z",
                        "startTime": "2026-07-02T11:55:00.000Z",
                        "endTime": "2026-07-02T12:23:00.000Z",
                        "runtime": "28 min",
                        "releaseInfo": "2022",
                        "genres": ["Anime", "Comédia"],
                        "cast": ["Takuya Eguchi", "Saori Hayami"],
                        "directors": ["Kazuhiro Furuhashi"],
                        "links": [],
                        "streams": [
                            {
                                "name": "PureTV",
                                "description": "AXN",
                                "url": "https://cdn.example.com/axn/master.m3u8",
                                "behaviorHints": { "notWebReady": true }
                            }
                        ]
                    }
                ]
            },
            {
                "id": "pure:amc",
                "type": "tv",
                "name": "AMC",
                "logo": "https://addon.example.com/logos/amc.png",
                "posterShape": "landscape",
                "behaviorHints": { "hasScheduledVideos": true },
                "videos": [
                    {
                        "id": "pure:amc:7q1z8r",
                        "title": "Tomb Raider - A Origem",
                        "overview": "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore.",
                        "thumbnail": "https://addon.example.com/thumbs/tomb-raider.jpg",
                        "released": "2026-07-02T11:25:00.000Z",
                        "startTime": "2026-07-02T11:25:00.000Z",
                        "endTime": "2026-07-02T13:35:00.000Z",
                        "runtime": "130 min",
                        "releaseInfo": "2018",
                        "genres": ["Ação", "Aventura"],
                        "cast": ["Alicia Vikander", "Dominic West"],
                        "directors": ["Roar Uthaug"],
                        "links": []
                    }
                ]
            }
        ],
        "cacheMaxAge": 300,
        "staleRevalidate": 1800,
        "staleError": 604800
    });

    let response = serde_json::from_value::<ResourceResponse>(json)
        .expect("the example must deserialize as a valid ResourceResponse");
    let metas_detailed = match &response {
        ResourceResponse::MetasDetailed { metas_detailed } => metas_detailed,
        _ => panic!("the example must parse as MetasDetailed"),
    };

    assert_eq!(metas_detailed.len(), 2);
    let axn = &metas_detailed[0];
    assert_eq!(axn.preview.id, "pure:axn");
    assert!(axn.preview.behavior_hints.has_scheduled_videos);
    assert_eq!(axn.videos.len(), 2);

    // NOTE: core re-sorts MetaItem.videos by `released` DESCENDING on
    // deserialization (VideoSortedVecAdapter) - the LiveTvGuide model
    // re-sorts shows by startTime ascending when deriving channels
    let spy_x_family = &axn.videos[0];
    let epg_info = spy_x_family
        .epg_info
        .as_ref()
        .expect("shows must parse with epg info");
    assert_eq!(epg_info.runtime.as_deref(), Some("28 min"));
    assert_eq!(epg_info.release_info.as_deref(), Some("2022"));
    assert_eq!(epg_info.genres, vec!["Anime", "Comédia"]);
    // "now" is 12:00 - S.W.A.T. has ended, Spy x Family is on air
    let now = Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap();
    assert!(!axn.videos[1].epg_info.as_ref().unwrap().is_live(now));
    assert!(epg_info.is_live(now));
    // the live show carries the channel's live stream for the Play button
    assert_eq!(spy_x_family.streams.len(), 1);
}
