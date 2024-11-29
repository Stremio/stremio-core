use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use assert_matches::assert_matches;
use chrono::{TimeZone, Utc};
use enclose::enclose;
use futures::future;
use once_cell::sync::Lazy;
use semver::Version;
use serde::Deserialize;
use url::Url;

use stremio_derive::Model;

use crate::{
    constants::{CATALOG_RESOURCE_NAME, LAST_VIDEOS_IDS_EXTRA_PROP},
    models::{
        ctx::Ctx,
        player::{Player, Selected as PlayerSelected},
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad, ActionPlayer, Event},
        Env, EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture,
    },
    types::{
        addon::{
            Descriptor, Manifest, ManifestCatalog, ManifestExtra, ResourcePath, ResourceRequest,
            ResourceResponse,
        },
        events::DismissedEventsBucket,
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        notifications::{NotificationItem, NotificationsBucket},
        profile::Profile,
        resource::{
            MetaItem, MetaItemId, MetaItemPreview, PosterShape, SeriesInfo, Stream, StreamSource,
            Video, VideoId,
        },
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
    unit_tests::{
        default_fetch_handler, Request, TestEnv, EVENTS, FETCH_HANDLER, NOW, REQUESTS, STATES,
    },
};

pub const PULL_NOTIFICATIONS_TEST_DATA: &[u8] = include_bytes!("./pull_notifications_data.json");

#[derive(Deserialize)]
struct TestData {
    network_requests: HashMap<String, ResourceResponse>,
    addons: Vec<Descriptor>,
    library_items: Vec<LibraryItem>,
    notification_items: Vec<NotificationItem>,
    result: HashMap<MetaItemId, HashMap<VideoId, NotificationItem>>,
}

#[test]
fn test_pull_notifications_and_play_in_player() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    /// Addon 1 with lastVideosIds catalog
    pub static ADDON_1: Lazy<Descriptor> = Lazy::new(|| Descriptor {
        manifest: Manifest {
            id: "addon_1".to_owned(),
            version: Version::new(0, 0, 1),
            name: "Addon 1".to_owned(),
            contact_email: None,
            description: None,
            logo: None,
            background: None,
            types: vec!["series".into()],
            resources: vec![CATALOG_RESOURCE_NAME.into()],
            id_prefixes: Some(vec!["tt".to_owned()]),
            catalogs: vec![ManifestCatalog {
                id: "lastVideosIds".to_owned(),
                r#type: "series".to_owned(),
                name: None,
                extra: ManifestExtra::Full {
                    props: vec![LAST_VIDEOS_IDS_EXTRA_PROP.to_owned()],
                },
            }],
            addon_catalogs: vec![],
            behavior_hints: Default::default(),
        },
        transport_url: Url::parse("https://addon_1.com/manifest.json").unwrap(),
        flags: Default::default(),
    });

    /// Meta item 1 with id `tt1` and 7 episodes from season 1
    pub static META_ITEM_1: Lazy<MetaItem> = Lazy::new(|| MetaItem {
        preview: MetaItemPreview {
            id: "tt1".to_string(),
            name: "name".to_string(),
            r#type: "series".to_string(),
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
            behavior_hints: Default::default(),
        },
        videos: vec![
            Video {
                id: "tt1:1:4".to_owned(),
                title: "ep4".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2019, 12, 20, 0, 0, 0).unwrap()),
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 4,
                }),
                trailer_streams: vec![],
            },
            Video {
                id: "tt1:1:5".to_owned(),
                title: "ep5".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 5,
                }),
                trailer_streams: vec![],
            },
            Video {
                id: "tt1:1:6".to_owned(),
                title: "ep6".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 5, 0, 0, 0).unwrap()),
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 6,
                }),
                trailer_streams: vec![],
            },
            Video {
                id: "tt1:1:7".to_owned(),
                title: "ep7".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 15, 0, 0, 0).unwrap()),
                overview: None,
                thumbnail: None,
                streams: vec![],
                series_info: Some(SeriesInfo {
                    season: 1,
                    episode: 7,
                }),
                trailer_streams: vec![],
            },
        ],
    });

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        let meta_item = META_ITEM_1.clone();
        match request {
            Request { url, method, .. }
                if url == "https://addon_1.com/catalog/series/lastVideosIds/lastVideosIds=tt1.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::MetasDetailed {
                    metas_detailed: vec![meta_item],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, method, .. }
                if url == "https://addon_1.com/meta/series/tt1.json" && method == "GET" =>
            {
                future::ok(
                    Box::new(ResourceResponse::Meta { meta: meta_item }) as Box<dyn Any + Send>
                )
                .boxed_env()
            }
            Request { url, method, .. }
                if url == "https://addon_1.com/stream/series/tt1%3A1%3A7.json"
                    && method == "GET" =>
            {
                future::ok(
                    Box::new(ResourceResponse::Streams { streams: vec![] }) as Box<dyn Any + Send>
                )
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2024, 1, 1, 10, 30, 0).unwrap();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    addons: vec![ADDON_1.clone()],
                    ..Default::default()
                },
                LibraryBucket::new(
                    None,
                    vec![LibraryItem {
                        id: "tt1".to_string(),
                        name: "name".to_string(),
                        r#type: "series".to_string(),
                        poster: None,
                        poster_shape: PosterShape::Poster,
                        removed: false,
                        temp: false,
                        ctime: Some(TestEnv::now()),
                        mtime: TestEnv::now(),
                        state: LibraryItemState {
                            watched: None,
                            time_watched: 1000,
                            overall_time_watched: 15 * 60 * 1000 + 1,
                            // Episode 5 is released on this date and we've watched it the later that day
                            last_watched: Some(Utc.with_ymd_and_hms(2020, 1, 1, 20, 0, 0).unwrap()),
                            times_watched: 5,
                            flagged_watched: 1,
                            time_offset: 100,
                            duration: 101,
                            video_id: Some("tt1:1:5".to_string()),
                            no_notif: false,
                        },
                        behavior_hints: Default::default(),
                    }],
                ),
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
            player: Default::default(),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::PullNotifications),
        })
    });
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );

    assert_eq!(
        runtime.model().unwrap().ctx.notifications.items.len(),
        1,
        "1 MetaItem should be in the notifications bucket"
    );

    let meta_notifs = runtime
        .model()
        .unwrap()
        .ctx
        .notifications
        .items
        .get("tt1")
        .expect("Should have new notifications for this MetaItem")
        .clone();

    assert_eq!(2, meta_notifs.len(), "Should have 2 video notifications");
    assert!(
        meta_notifs.contains_key("tt1:1:6"),
        "Should have notification for tt1:1:6"
    );
    assert!(
        meta_notifs.contains_key("tt1:1:7"),
        "Should have notification for tt1:1:7"
    );
    // Start watching episode 6
    // This should dismiss all notifications for this MetaItem Id.
    // libraryItem.state.lastWatched dismisses any unwatched episode notifications
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(PlayerSelected {
                stream: Stream {
                    source: StreamSource::Url {
                        url: Url::parse("https://example.com/stream.mp4").unwrap(),
                    },
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: Default::default(),
                },
                meta_request: Some(ResourceRequest {
                    base: Url::parse("https://addon_1.com/manifest.json").unwrap(),
                    path: ResourcePath {
                        id: "tt1".to_owned(),
                        resource: "meta".to_owned(),
                        r#type: "series".to_owned(),
                        extra: vec![],
                    },
                }),
                stream_request: Some(ResourceRequest {
                    base: Url::parse("https://addon_1.com/manifest.json").unwrap(),
                    path: ResourcePath {
                        resource: "stream".to_owned(),
                        r#type: "series".to_owned(),
                        id: "tt1:1:6".to_owned(),
                        extra: vec![],
                    },
                }),
                subtitles_path: None,
            }))),
        });
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::TimeChanged {
                time: 95,
                duration: 100,
                device: "chromecast".to_owned(),
            }),
        });
    });

    assert!(
        !runtime
            .model()
            .unwrap()
            .ctx
            .notifications
            .items
            .contains_key("tt1"),
        "All notifications for this MetaItem should be now dismissed"
    );
    // before pulling notifications, make sure to update the last_updated time
    *NOW.write().unwrap() = Utc::now();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::PullNotifications),
        })
    });

    assert_eq!(
        runtime.model().unwrap().ctx.notifications.items.len(),
        0,
        "MetaItem Id should be removed because it doesn't have any notifications anymore"
    );
}

#[test]
fn test_pull_notifications_test_cases() {
    let tests = serde_json::from_slice::<Vec<TestData>>(PULL_NOTIFICATIONS_TEST_DATA).unwrap();
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    for test in tests {
        let _env_lock = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
        let fetch_handler = enclose!((test.network_requests => network_requests) move |request: Request| -> TryEnvFuture<Box<dyn Any + Send>> {
            if let Some(result) = network_requests.get(&request.url) {
                return future::ok(Box::new(result.to_owned()) as Box<dyn Any + Send>).boxed_env();
            }

            default_fetch_handler(request)
        });
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

        let (runtime, _rx) = Runtime::<TestEnv, _>::new(
            TestModel {
                ctx: Ctx::new(
                    Profile {
                        addons: test.addons,
                        ..Default::default()
                    },
                    LibraryBucket::new(None, test.library_items),
                    StreamsBucket::default(),
                    ServerUrlsBucket::new::<TestEnv>(None),
                    NotificationsBucket::new::<TestEnv>(None, test.notification_items),
                    SearchHistoryBucket::default(),
                    DismissedEventsBucket::default(),
                ),
            },
            vec![],
            1000,
        );
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::PullNotifications),
            })
        });

        pretty_assertions::assert_eq!(
            runtime.model().unwrap().ctx.notifications.items,
            test.result,
            "Notifications items match"
        );
    }
}

#[test]
fn test_dismiss_notification() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    let _env_lock = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    ..Default::default()
                },
                LibraryBucket::new(
                    None,
                    vec![
                        LibraryItem {
                            id: "tt1".to_string(),
                            name: "Item 1".to_string(),
                            r#type: "series".to_string(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: false,
                            temp: false,
                            ctime: Some(Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap()),
                            mtime: Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                            state: LibraryItemState {
                                last_watched: Some(
                                    Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                                ),
                                time_watched: 40 * 60 * 60 * 1000,
                                time_offset: 15,
                                overall_time_watched: 140 * 60 * 60 * 1000,
                                times_watched: 2,
                                flagged_watched: 1,
                                duration: 55 * 60 * 60 * 1000,
                                video_id: Some("tt1:1".into()),
                                watched: None,
                                no_notif: false,
                            },
                            behavior_hints: Default::default(),
                        },
                        LibraryItem {
                            id: "tt2".to_string(),
                            name: "Item 2".to_string(),
                            r#type: "series".to_string(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: false,
                            temp: false,
                            ctime: Some(Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap()),
                            mtime: Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                            state: LibraryItemState {
                                last_watched: Some(
                                    Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                                ),
                                time_watched: 40 * 60 * 60 * 1000,
                                time_offset: 15,
                                overall_time_watched: 140 * 60 * 60 * 1000,
                                times_watched: 2,
                                flagged_watched: 1,
                                duration: 55 * 60 * 60 * 1000,
                                video_id: Some("tt1:1".into()),
                                watched: None,
                                no_notif: false,
                            },
                            behavior_hints: Default::default(),
                        },
                    ],
                ),
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(
                    None,
                    vec![
                        NotificationItem {
                            meta_id: "tt1".to_string(),
                            video_id: "tt1:2".to_string(),
                            video_released: Utc.with_ymd_and_hms(2023, 7, 10, 0, 0, 0).unwrap(),
                        },
                        NotificationItem {
                            meta_id: "tt2".to_string(),
                            video_id: "tt2:10".to_string(),
                            video_released: Utc.with_ymd_and_hms(2023, 8, 14, 0, 0, 0).unwrap(),
                        },
                    ],
                ),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    // update now to later date than both last_watched
    let expected_last_watched = Utc.with_ymd_and_hms(2023, 8, 14, 0, 0, 0).unwrap();
    *NOW.write().unwrap() = expected_last_watched;

    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::DismissNotificationItem("tt1".into())),
            })
        }),
    );
    let events = EVENTS.read().unwrap();
    assert_eq!(events.len(), 5);

    let events = events
        .iter()
        .map(|event| {
            event
                .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
                .unwrap()
        })
        .collect::<Vec<_>>();

    assert_matches!(
        events[0],
        RuntimeEvent::NewState(fields, model) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Ctx && model.ctx.notifications.items.len() == 1,
        "We should have notifications for 1 LibraryItem ids"
    );
    assert_matches!(
        events[1],
        RuntimeEvent::NewState(fields, model) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Ctx && model.ctx.notifications.items.len() == 1,
        "We should have notifications for 1 LibraryItem ids"
    );

    assert_matches!(
        events[2],
        RuntimeEvent::CoreEvent(crate::runtime::msg::Event::NotificationsDismissed {
            id
        }) if id == "tt1"
    );
    assert_matches!(
        events[3],
        RuntimeEvent::CoreEvent(crate::runtime::msg::Event::NotificationsPushedToStorage {
            ids
        }) if ids == &["tt2".to_string()]
    );
    assert_matches!(
        events[4],
        RuntimeEvent::CoreEvent(Event::LibraryItemsPushedToStorage { ids }) if ids == &["tt1".to_string()],
        "LibraryItem should be pushed to storage"
    );

    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(3, states.len());

    // state 0 - we have both notifs.
    {
        let notification_items = &states[0].ctx.notifications.items;
        assert_eq!(
            2,
            notification_items.len(),
            "Should have both notifications"
        );
    }

    // state 1 - we have only the remaining notification after we've dismissed the other
    {
        let notification_items = &states[1].ctx.notifications.items;
        assert_eq!(
            1,
            notification_items.len(),
            "Should have single notification"
        );

        let notification_library_item = notification_items
            .get("tt2")
            .expect("Should have notification");
        assert!(
            notification_library_item.get("tt2:10").is_some(),
            "Should have notification"
        );
    }

    // state 2 - `LibraryItem.state.last_watched` of the dismissed item's notifications should be updated too
    // by the `UpdateLibraryItem`
    {
        assert_eq!(
            Some(expected_last_watched),
            states[2]
                .ctx
                .library
                .items
                .get("tt1")
                .unwrap()
                .state
                .last_watched
        );
    }
}
