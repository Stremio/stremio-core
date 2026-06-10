use crate::{
    constants::{CINEMETA_URL, META_RESOURCE_NAME, OFFICIAL_ADDONS},
    models::{
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionMetaDetails},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{ResourcePath, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, SeriesInfo, Video},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};
use chrono::{DateTime, Utc};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[derive(Model, Default, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    meta_details: MetaDetails,
}

const PREVIOUS_TIME_WATCHED: u64 = 40 * 60 * 1000;
const PREVIOUS_OVERALL_TIME_WATCHED: u64 = 5 * 60 * 1000;

fn create_video(season: u32, episode: u32) -> Video {
    Video {
        id: format!("tt123456:{season}:{episode}"),
        title: format!("S{season}E{episode}"),
        series_info: Some(SeriesInfo { season, episode }),
        ..Default::default()
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://v3-cinemeta.strem.io/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt123456".to_owned(),
                        r#type: "series".to_owned(),
                        ..Default::default()
                    },
                    videos: vec![create_video(1, 1), create_video(1, 2)],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

fn create_library_item(video_id: &str) -> LibraryItem {
    LibraryItem {
        id: "tt123456".to_owned(),
        name: "Test Series".to_owned(),
        r#type: "series".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: DateTime::<Utc>::default(),
        state: LibraryItemState {
            video_id: Some(video_id.to_owned()),
            time_offset: PREVIOUS_TIME_WATCHED,
            time_watched: PREVIOUS_TIME_WATCHED,
            overall_time_watched: PREVIOUS_OVERALL_TIME_WATCHED,
            flagged_watched: 1,
            duration: 60 * 60 * 1000,
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

#[test]
fn mark_video_as_watched_advances_video_id() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item("tt123456:1:1");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: OFFICIAL_ADDONS
                        .iter()
                        .filter(|addon| addon.transport_url == *CINEMETA_URL)
                        .cloned()
                        .collect(),
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".to_owned(), library_item)]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: ResourcePath {
                    resource: META_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: "tt123456".to_owned(),
                    extra: vec![],
                },
                stream_path: None,
                guess_stream: false,
            })),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::MetaDetails(ActionMetaDetails::MarkVideoAsWatched(
                create_video(1, 1),
                true,
            )),
        });
    });

    let model = runtime.model().unwrap();
    let library_item = model.ctx.library.items.get("tt123456").unwrap();
    assert_eq!(
        library_item.state.video_id,
        Some("tt123456:1:2".to_owned()),
        "video_id should advance to the next episode after marking current as watched",
    );
    assert_eq!(
        library_item.state.time_offset, 1,
        "time_offset should be reset when advancing to the next episode",
    );
    assert_eq!(
        library_item.state.time_watched, 0,
        "time_watched should not carry over to the next episode",
    );
    assert_eq!(
        library_item.state.flagged_watched, 0,
        "flagged_watched should be reset for the next episode",
    );
    assert_eq!(
        library_item.state.overall_time_watched,
        PREVIOUS_OVERALL_TIME_WATCHED + PREVIOUS_TIME_WATCHED,
        "previous episode time_watched should be folded into overall_time_watched",
    );
}

#[test]
fn mark_last_video_as_watched_clears_continue_watching_progress() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item("tt123456:1:2");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: OFFICIAL_ADDONS
                        .iter()
                        .filter(|addon| addon.transport_url == *CINEMETA_URL)
                        .cloned()
                        .collect(),
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".to_owned(), library_item)]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: ResourcePath {
                    resource: META_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: "tt123456".to_owned(),
                    extra: vec![],
                },
                stream_path: None,
                guess_stream: false,
            })),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::MetaDetails(ActionMetaDetails::MarkVideoAsWatched(
                create_video(1, 2),
                true,
            )),
        });
    });

    let model = runtime.model().unwrap();
    let library_item = model.ctx.library.items.get("tt123456").unwrap();
    assert_eq!(
        library_item.state.video_id,
        Some("tt123456:1:2".to_owned()),
        "video_id should remain on the last episode when there is no next video",
    );
    assert_eq!(
        library_item.state.time_offset, 0,
        "time_offset should be cleared when there is no next episode",
    );
}
