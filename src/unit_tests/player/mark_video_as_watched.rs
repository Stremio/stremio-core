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
        epg_info: None,
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

fn fetch_handler_next_season(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://transport_url/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt123456".to_owned(),
                        r#type: "series".to_owned(),
                        ..Default::default()
                    },
                    videos: vec![create_video(1, 1), create_video(1, 2), create_video(2, 1)],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => fetch_handler_s1e1_current(request),
    }
}

fn dispatch_time_changed(runtime: &Runtime<TestEnv, TestModel>, time: u64) {
    dispatch_time_changed_with_duration(runtime, time, 3_600_000);
}

fn run_with_library_item(library_item: LibraryItem, run: impl FnOnce(Runtime<TestEnv, TestModel>)) {
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".into(), library_item)]
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
    run(runtime);
}

fn load_video(runtime: &Runtime<TestEnv, TestModel>, video_id: &str) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: create_stream(),
                stream_request: Some(make_stream_request(video_id)),
                meta_request: Some(make_meta_request()),
                subtitles_path: None,
            }))),
        });
    });
}

fn dispatch_time_changed_with_duration(
    runtime: &Runtime<TestEnv, TestModel>,
    time: u64,
    duration: u64,
) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::TimeChanged {
                time,
                duration,
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
fn mark_season_as_watched_clears_resume_on_unload_without_double_counting_playback() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_s1e1_current);

    let mut library_item = make_library_item("tt123456:1:1");
    library_item.state.time_offset = 600_000;
    library_item.state.time_watched = 600_000;
    library_item.state.overall_time_watched = 120_000;
    library_item.state.flagged_watched = 1;
    library_item.state.duration = 3_600_000;

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt123456".into(), library_item)]
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
            action: Action::Player(ActionPlayer::MarkSeasonAsWatched(1, true)),
        });
    });

    {
        let model = runtime.model().unwrap();
        let library_item = model.ctx.library.items.get("tt123456").unwrap();
        assert_eq!(library_item.state.time_offset, 600_000);
        assert_eq!(library_item.state.time_watched, 600_000);
    }
    dispatch_time_changed(&runtime, 700_000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });

    let model = runtime.model().unwrap();
    let library_item = model.ctx.library.items.get("tt123456").unwrap();
    assert_eq!(
        library_item.state.time_offset, 0,
        "player season watched action should clear stale resume progress when no released episode remains",
    );
    assert_eq!(library_item.state.time_watched, 700_000);
    assert_eq!(library_item.state.overall_time_watched, 220_000);
    drop(model);

    load_video(&runtime, "tt123456:1:1");
    dispatch_time_changed(&runtime, 10_000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });
    let model = runtime.model().unwrap();
    assert_eq!(
        model
            .ctx
            .library
            .items
            .get("tt123456")
            .unwrap()
            .state
            .time_offset,
        10_000,
        "starting a rewatch should create fresh resume progress",
    );
}

#[test]
fn mark_season_as_watched_advances_on_unload_and_survives_stream_reload() {
    for reload_stream in [false, true] {
        let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_next_season);
        run_with_library_item(make_library_item("tt123456:1:1"), |runtime| {
            load_video(&runtime, "tt123456:1:1");
            dispatch_time_changed(&runtime, 600_000);
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Player(ActionPlayer::MarkSeasonAsWatched(1, true)),
                });
            });
            if reload_stream {
                let mut stream = create_stream();
                stream.source = StreamSource::Url {
                    url: "https://other_source_url".parse().unwrap(),
                };
                TestEnv::run(|| {
                    runtime.dispatch(RuntimeAction {
                        field: None,
                        action: Action::Load(ActionLoad::Player(Box::new(Selected {
                            stream,
                            stream_request: Some(make_stream_request("tt123456:1:1")),
                            meta_request: Some(make_meta_request()),
                            subtitles_path: None,
                        }))),
                    });
                });
            }
            dispatch_time_changed(&runtime, 700_000);
            {
                let model = runtime.model().unwrap();
                let library_item = model.player.library_item.as_ref().unwrap();
                assert_eq!(library_item.state.video_id.as_deref(), Some("tt123456:1:1"));
                assert_eq!(library_item.state.time_offset, 700_000);
                assert_eq!(library_item.state.time_watched, 700_000);
            }
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Unload,
                });
            });
            let model = runtime.model().unwrap();
            let library_item = model.ctx.library.items.get("tt123456").unwrap();
            assert_eq!(
                library_item.state.video_id.as_deref(),
                Some("tt123456:2:1"),
                "advance after reloading the stream: {reload_stream}",
            );
            assert_eq!(library_item.state.time_offset, 1);
            assert_eq!(library_item.state.time_watched, 0);
        });
    }
}

#[test]
fn marking_season_or_current_video_unwatched_cancels_deferred_resume_cleanup() {
    for action in [
        ActionPlayer::MarkSeasonAsWatched(1, false),
        ActionPlayer::MarkVideoAsWatched(create_video(1, 1), false),
    ] {
        let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_next_season);
        run_with_library_item(make_library_item("tt123456:1:1"), |runtime| {
            load_video(&runtime, "tt123456:1:1");
            dispatch_time_changed(&runtime, 600_000);
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Player(ActionPlayer::MarkSeasonAsWatched(1, true)),
                });
            });
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Player(action),
                });
            });
            dispatch_time_changed(&runtime, 700_000);
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Unload,
                });
            });
            let model = runtime.model().unwrap();
            let library_item = model.ctx.library.items.get("tt123456").unwrap();
            assert_eq!(library_item.state.video_id.as_deref(), Some("tt123456:1:1"));
            assert_eq!(library_item.state.time_offset, 700_000);
            assert_eq!(library_item.state.time_watched, 700_000);
        });
    }
}

#[test]
fn mark_season_as_watched_reconciles_before_loading_another_video() {
    for same_series in [false, true] {
        let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_next_season);
        run_with_library_item(make_library_item("tt123456:1:1"), |runtime| {
            load_video(&runtime, "tt123456:1:1");
            dispatch_time_changed(&runtime, 600_000);
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Player(ActionPlayer::MarkSeasonAsWatched(1, true)),
                });
            });
            dispatch_time_changed(&runtime, 700_000);
            TestEnv::run(|| {
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Load(ActionLoad::Player(Box::new(Selected {
                        stream: create_stream(),
                        stream_request: same_series.then(|| make_stream_request("tt123456:2:1")),
                        meta_request: same_series.then(make_meta_request),
                        subtitles_path: None,
                    }))),
                });
            });
            let model = runtime.model().unwrap();
            let library_item = model.ctx.library.items.get("tt123456").unwrap();
            assert_eq!(library_item.state.video_id.as_deref(), Some("tt123456:2:1"));
            assert_eq!(library_item.state.time_offset, 1);
            drop(model);
            if same_series {
                dispatch_time_changed(&runtime, 100_000);
                TestEnv::run(|| {
                    runtime.dispatch(RuntimeAction {
                        field: None,
                        action: Action::Unload,
                    });
                });
                let model = runtime.model().unwrap();
                let library_item = model.ctx.library.items.get("tt123456").unwrap();
                assert_eq!(library_item.state.time_offset, 100_000);
            }
        });
    }
}

#[test]
fn mark_other_season_as_watched_preserves_resume_progress() {
    for (video_id, season) in [("tt123456:2:1", 1), ("tt123456:1:1", 2)] {
        let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_next_season);

        let mut library_item = make_library_item(video_id);
        library_item.state.time_offset = 600_000;
        library_item.state.time_watched = 600_000;
        library_item.state.duration = 3_600_000;
        let videos = vec![create_video(1, 1), create_video(1, 2), create_video(2, 1)];
        let mut watched = library_item.state.watched_bitfield(&videos);
        watched.set_video(video_id, true);
        library_item.state.watched = Some(watched.into());

        let (runtime, _rx) = Runtime::<TestEnv, _>::new(
            TestModel {
                ctx: Ctx {
                    library: LibraryBucket {
                        uid: None,
                        items: vec![("tt123456".into(), library_item)]
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
                    stream_request: Some(make_stream_request(video_id)),
                    meta_request: Some(make_meta_request()),
                    subtitles_path: None,
                }))),
            });
        });
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Player(ActionPlayer::MarkSeasonAsWatched(season, true)),
            });
        });

        let model = runtime.model().unwrap();
        let library_item = model.ctx.library.items.get("tt123456").unwrap();
        assert_eq!(library_item.state.video_id.as_deref(), Some(video_id));
        assert_eq!(library_item.state.time_offset, 600_000);
        assert_eq!(library_item.state.time_watched, 600_000);
        let watched = library_item.state.watched_bitfield(&videos);
        assert!(videos
            .iter()
            .filter(|video| video.series_info.as_ref().unwrap().season == season)
            .all(|video| watched.get_video(&video.id)));
    }
}

#[test]
fn time_changed_without_duration_does_not_flag_watched() {
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

    // live streams report no duration
    dispatch_time_changed_with_duration(&runtime, 600_000, 0);

    let model = runtime.model().unwrap();
    let library_item = model.player.library_item.as_ref().unwrap();
    assert_eq!(
        library_item.state.flagged_watched, 0,
        "playback without a duration must not cross the watched threshold",
    );
    assert_eq!(
        library_item.state.times_watched, 0,
        "playback without a duration must not increase times_watched",
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

fn dispatch_seek(runtime: &Runtime<TestEnv, TestModel>, time: u64, duration: u64) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::Seek {
                time,
                duration,
                device: "test_device".to_owned(),
            }),
        });
    });
}

fn dispatch_ended(runtime: &Runtime<TestEnv, TestModel>) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::Ended),
        });
    });
}

fn make_movie_library_item() -> LibraryItem {
    LibraryItem {
        id: "tt654321".into(),
        name: "Test Movie".into(),
        r#type: "movie".into(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: DateTime::<Utc>::default(),
        state: LibraryItemState {
            video_id: Some("tt654321".to_owned()),
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

fn make_movie_meta_request() -> ResourceRequest {
    ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: META_RESOURCE_NAME.to_owned(),
            r#type: "movie".to_owned(),
            id: "tt654321".to_owned(),
            extra: vec![],
        },
    }
}

fn make_movie_stream_request() -> ResourceRequest {
    ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: STREAM_RESOURCE_NAME.to_owned(),
            r#type: "movie".to_owned(),
            id: "tt654321".to_owned(),
            extra: vec![],
        },
    }
}

fn fetch_handler_movie(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://transport_url/meta/movie/tt654321.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt654321".to_owned(),
                        r#type: "movie".to_owned(),
                        name: "Test Movie".to_owned(),
                        ..Default::default()
                    },
                    videos: vec![],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

#[test]
fn seek_into_credits_then_ended_marks_series_episode_watched_before_advancing() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_s1e1_current);

    run_with_library_item(make_library_item("tt123456:1:1"), |runtime| {
        load_video(&runtime, "tt123456:1:1");
        dispatch_seek(&runtime, 3_500_000, 3_600_000);

        {
            let model = runtime.model().unwrap();
            let item = model.player.library_item.as_ref().unwrap();
            assert_eq!(
                item.state.times_watched, 0,
                "seek alone must not mark watched"
            );
            assert_eq!(
                item.state.flagged_watched, 0,
                "seek alone must not flag watched"
            );
        }

        dispatch_ended(&runtime);

        {
            let model = runtime.model().unwrap();
            let item = model.player.library_item.as_ref().unwrap();
            let videos = vec![create_video(1, 1), create_video(1, 2)];
            let watched = item.state.watched_bitfield(&videos);
            assert!(watched.get_video("tt123456:1:1"));
            assert_eq!(item.state.times_watched, 1);
            assert_eq!(item.state.flagged_watched, 1);
        }

        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Unload,
            });
        });

        let model = runtime.model().unwrap();
        let item = model.ctx.library.items.get("tt123456").unwrap();
        let videos = vec![create_video(1, 1), create_video(1, 2)];
        let watched = item.state.watched_bitfield(&videos);
        assert!(watched.get_video("tt123456:1:1"));
        assert_eq!(item.state.video_id.as_deref(), Some("tt123456:1:2"));
        assert_eq!(item.state.time_offset, 1);
    });
}

#[test]
fn seek_into_credits_then_ended_marks_movie_watched_and_clears_resume() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_movie);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt654321".into(), make_movie_library_item())]
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
                stream_request: Some(make_movie_stream_request()),
                meta_request: Some(make_movie_meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    dispatch_seek(&runtime, 3_500_000, 3_600_000);
    {
        let model = runtime.model().unwrap();
        let item = model.player.library_item.as_ref().unwrap();
        assert_eq!(
            item.state.times_watched, 0,
            "seek alone must not mark watched"
        );
        assert_eq!(
            item.state.flagged_watched, 0,
            "seek alone must not flag watched"
        );
    }

    dispatch_ended(&runtime);
    {
        let model = runtime.model().unwrap();
        let item = model.player.library_item.as_ref().unwrap();
        assert_eq!(item.state.times_watched, 1);
        assert_eq!(item.state.flagged_watched, 1);
        assert!(item.state.last_watched.is_some());
    }

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Unload,
        });
    });

    let model = runtime.model().unwrap();
    let item = model.ctx.library.items.get("tt654321").unwrap();
    assert_eq!(item.state.time_offset, 0);
    assert_eq!(item.state.times_watched, 1);
    assert_eq!(item.state.flagged_watched, 1);
    assert!(!item.is_in_continue_watching());
}

#[test]
fn seek_into_credits_without_ended_does_not_mark_watched() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler_movie);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("tt654321".into(), make_movie_library_item())]
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
                stream_request: Some(make_movie_stream_request()),
                meta_request: Some(make_movie_meta_request()),
                subtitles_path: None,
            }))),
        });
    });

    dispatch_seek(&runtime, 3_500_000, 3_600_000);

    let model = runtime.model().unwrap();
    let item = model.player.library_item.as_ref().unwrap();
    assert_eq!(item.state.times_watched, 0);
    assert_eq!(item.state.flagged_watched, 0);
}
