use std::any::Any;

use chrono::{TimeZone, Utc};
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::META_RESOURCE_NAME,
    models::{ctx::Ctx, live_tv_continue_watching::LiveTvContinueWatching},
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestBehaviorHints, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW},
};

#[test]
fn live_tv_continue_watching_rewind_library_item() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_continue_watching: LiveTvContinueWatching,
    }

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match &request {
            Request { url, method, .. }
                if url == "https://addon/meta/tv/pure%3Aaxn.json" && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::Meta {
                    meta: MetaItem {
                        preview: MetaItemPreview {
                            id: "pure:axn".to_owned(),
                            r#type: "tv".to_owned(),
                            name: "AXN".to_owned(),
                            ..MetaItemPreview::default()
                        },
                        videos: vec![Video {
                            id: "pure:axn:1".to_owned(),
                            epg_info: Some(VideoEpgInfo {
                                start_time: Utc.with_ymd_and_hms(2026, 7, 2, 11, 0, 0).unwrap(),
                                end_time: Utc.with_ymd_and_hms(2026, 7, 2, 13, 0, 0).unwrap(),
                                runtime: None,
                                release_info: None,
                                genres: vec![],
                                cast: vec![],
                                directors: vec![],
                                links: vec![],
                                ratings: vec![],
                            }),
                            ..Video::default()
                        }],
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 11, 30, 0).unwrap();

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![Descriptor {
                        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
                        flags: Default::default(),
                        manifest: Manifest {
                            id: "addon".to_owned(),
                            types: vec!["tv".into()],
                            resources: vec![META_RESOURCE_NAME.into()],
                            id_prefixes: None,
                            behavior_hints: ManifestBehaviorHints {
                                epg_provider: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: [(
                        "pure:axn".into(),
                        LibraryItem {
                            id: "pure:axn".to_owned(),
                            name: "AXN".to_owned(),
                            r#type: "tv".to_owned(),
                            poster: None,
                            poster_shape: Default::default(),
                            removed: true,
                            temp: true,
                            ctime: None,
                            mtime: Utc.with_ymd_and_hms(2026, 7, 2, 11, 0, 0).unwrap(),
                            state: LibraryItemState {
                                last_watched: Some(
                                    Utc.with_ymd_and_hms(2026, 7, 2, 11, 0, 0).unwrap(),
                                ),
                                ..Default::default()
                            },
                            behavior_hints: Default::default(),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
                ..Default::default()
            },
            live_tv_continue_watching: Default::default(),
        },
        vec![],
        1000,
    );
    let dispatch = |action: Action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action,
            });
        });
    };

    dispatch(Action::Load(ActionLoad::LiveTvContinueWatching));
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .live_tv_continue_watching
            .items
            .len(),
        1,
        "the watched channel is in the row"
    );

    dispatch(Action::Ctx(ActionCtx::RewindLibraryItem(
        "pure:axn".to_owned(),
    )));
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get("pure:axn")
            .expect("library item should still exist")
            .state
            .last_watched,
        None,
        "rewinding a live channel clears last_watched"
    );
    assert!(
        runtime
            .model()
            .unwrap()
            .live_tv_continue_watching
            .items
            .is_empty(),
        "the dismissed channel is removed from the row"
    );
}
