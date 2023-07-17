use crate::constants::LAST_VIDEOS_IDS_EXTRA_PROP;
use crate::models::ctx::Ctx;
use crate::models::player::{Player, Selected as PlayerSelected};
use crate::runtime::msg::{Action, ActionCtx, ActionLoad, ActionPlayer};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::{
    Descriptor, Manifest, ManifestCatalog, ManifestExtra, ResourcePath, ResourceRequest,
    ResourceResponse,
};
use crate::types::library::{LibraryBucket, LibraryItem, LibraryItemState};
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::resource::{
    MetaItem, MetaItemPreview, PosterShape, SeriesInfo, Stream, StreamSource, Video,
};
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS};
use chrono::TimeZone;
use chrono::Utc;
use futures::future;
use once_cell::sync::Lazy;
use semver::Version;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[test]
fn pull_notifications() {
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
            types: vec![],
            resources: vec![],
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
    let _env_mutex = TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
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
                        temp: true,
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
                            last_video_released: Some(
                                Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
                            ),
                            notifications_disabled: false,
                        },
                        behavior_hints: Default::default(),
                    }],
                ),
                StreamsBucket::default(),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
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
        meta_notifs.get("tt1:1:6").is_some(),
        "Should have notification for tt1:1:6"
    );
    assert!(
        meta_notifs.get("tt1:1:7").is_some(),
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
                video_params: None,
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
        runtime
            .model()
            .unwrap()
            .ctx
            .notifications
            .items
            .get("tt1")
            .is_none(),
        "All notifications for this MetaItem should be now dismissed"
    );
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
