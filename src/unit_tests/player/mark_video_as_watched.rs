use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{ResourcePath, ResourceRequest, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        resource::{MetaItem, MetaItemPreview, SeriesInfo, Stream, StreamSource, Video},
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
    player: Player,
}

fn create_video(season: u32, episode: u32) -> Video {
    Video {
        id: format!("tt123456:{season}:{episode}"),
        title: format!("S{season}E{episode}"),
        released: None,
        overview: None,
        thumbnail: None,
        streams: vec![],
        series_info: Some(SeriesInfo { season, episode }),
        trailer_streams: vec![],
    }
}

fn create_stream() -> Stream {
    Stream {
        source: StreamSource::Url {
            url: "https://source_url".parse().unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    }
}

fn make_library_item(video_id: &str) -> LibraryItem {
    LibraryItem {
        id: "tt123456".into(),
        name: "Test Series".into(),
        r#type: "series".into(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        // Use the epoch so that E::now() (frozen by TestEnv::reset) is always >= mtime,
        // satisfying the merge_items guard (new_item.mtime >= item.mtime).
        mtime: DateTime::<Utc>::default(),
        state: LibraryItemState {
            video_id: Some(video_id.to_owned()),
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

fn make_meta_request() -> ResourceRequest {
    ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: META_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            id: "tt123456".to_owned(),
            extra: vec![],
        },
    }
}

fn make_stream_request(video_id: &str) -> ResourceRequest {
    ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: STREAM_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            id: video_id.to_owned(),
            extra: vec![],
        },
    }
}

// Fetch handler for meta [S1E1, S1E2] and the next-stream request for S1E2.
fn fetch_handler_s1e1_current(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://transport_url/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt123456".to_owned(),
                        r#type: "series".to_owned(),
                        ..Default::default()
                    },
                    // direct construction bypasses the SortedVec adapter, so order matters
                    videos: vec![create_video(1, 1), create_video(1, 2)],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        Request { url, .. }
            if url == "https://transport_url/stream/series/tt123456%3A1%3A2.json" =>
        {
            future::ok(
                Box::new(ResourceResponse::Streams { streams: vec![] }) as Box<dyn Any + Send>
            )
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

// Fetch handler for meta [S1E1, S1E2] when loading with S1E2 as current.
// No next video, so no next-stream fetch arm needed.
fn fetch_handler_s1e2_current(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://transport_url/meta/series/tt123456.json" => {
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

fn dispatch_time_changed(runtime: &Runtime<TestEnv, TestModel>, time: u64) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::TimeChanged {
                time,
                duration: 3_600_000,
                device: "test_device".to_owned(),
            }),
        });
    });
}

#[test]
fn mark_video_as_watched_advances_video_id_on_unload() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_s1e1_current);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".into(), make_library_item("tt123456:1:1"))]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            player: Player::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: create_stream(),
                stream_request: Some(make_stream_request("tt123456:1:1")),
                meta_request: Some(make_meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    dispatch_time_changed(&runtime, 600_000);

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::MarkVideoAsWatched(create_video(1, 1), true)),
        });
    });

    // playback continues after marking and must not lose the advancement
    dispatch_time_changed(&runtime, 700_000);

    assert_eq!(
        runtime
            .model()
            .unwrap()
            .player
            .library_item
            .as_ref()
            .unwrap()
            .state
            .video_id,
        Some("tt123456:1:1".to_owned()),
        "video_id should not advance while the video is still playing",
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });

    let model = runtime.model().unwrap();
    let library_item = model.ctx.library.items.get("tt123456").unwrap();
    assert_eq!(
        library_item.state.video_id,
        Some("tt123456:1:2".to_owned()),
        "video_id should advance to the next episode on unload after marking current as watched",
    );
    assert_eq!(
        library_item.state.time_offset, 1,
        "time_offset should be reset when advancing to the next episode",
    );
}

#[test]
fn mark_last_episode_as_watched_does_not_advance() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_s1e2_current);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".into(), make_library_item("tt123456:1:2"))]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            player: Player::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: create_stream(),
                stream_request: Some(make_stream_request("tt123456:1:2")),
                meta_request: Some(make_meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    dispatch_time_changed(&runtime, 600_000);

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::MarkVideoAsWatched(create_video(1, 2), true)),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
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
        "time_offset should be reset when the last episode is marked as watched",
    );
}

#[test]
fn mark_video_as_unwatched_does_not_advance_video_id() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_s1e1_current);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".into(), make_library_item("tt123456:1:1"))]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
            player: Player::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: create_stream(),
                stream_request: Some(make_stream_request("tt123456:1:1")),
                meta_request: Some(make_meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::MarkVideoAsWatched(create_video(1, 1), true)),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::MarkVideoAsWatched(create_video(1, 1), false)),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });

    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get("tt123456")
            .unwrap()
            .state
            .video_id,
        Some("tt123456:1:1".to_owned()),
        "video_id must not change when the video is marked as unwatched before unload",
    );
}
