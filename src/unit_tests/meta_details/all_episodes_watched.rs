use std::any::Any;

use chrono::{DateTime, Duration, Utc};
use futures::future;
use stremio_derive::Model;
use stremio_watched_bitfield::WatchedBitField;

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

#[derive(Model, Default, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    meta_details: MetaDetails,
}

const META_ID: &str = "tt123456";

fn create_video(season: u32, episode: u32) -> Video {
    Video {
        id: format!("{META_ID}:{season}:{episode}"),
        title: format!("S{season}E{episode}"),
        series_info: Some(SeriesInfo { season, episode }),
        ..Default::default()
    }
}

fn unreleased_video(season: u32, episode: u32) -> Video {
    Video {
        released: Some(Utc::now() + Duration::days(365)),
        ..create_video(season, episode)
    }
}

/// S0E1 (special), S1E1, S1E2 and an unreleased S1E3
fn videos() -> Vec<Video> {
    vec![
        create_video(0, 1),
        create_video(1, 1),
        create_video(1, 2),
        unreleased_video(1, 3),
    ]
}

fn meta_item() -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: META_ID.to_owned(),
            r#type: "series".to_owned(),
            ..Default::default()
        },
        videos: videos(),
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://v3-cinemeta.strem.io/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta { meta: meta_item() }) as Box<dyn Any + Send>)
                .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

fn create_library_item(r#type: &str, state: LibraryItemState) -> LibraryItem {
    LibraryItem {
        id: META_ID.to_owned(),
        name: "Test Series".to_owned(),
        r#type: r#type.to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: DateTime::<Utc>::default(),
        state,
        behavior_hints: Default::default(),
    }
}

/// Bitfield with the given episodes ticked
fn watched_bitfield(watched_ids: &[&str]) -> WatchedBitField {
    let mut bitfield = LibraryItemState::default().watched_bitfield(&videos());
    for id in watched_ids {
        bitfield.set_video(id, true);
    }
    bitfield
}

fn run_with_library_item<R>(
    library_item: LibraryItem,
    run: impl FnOnce(Runtime<TestEnv, TestModel>) -> R,
) -> R {
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
                    items: vec![(META_ID.to_owned(), library_item)]
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
    run(runtime)
}

fn load_meta_details(runtime: &Runtime<TestEnv, TestModel>) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: ResourcePath {
                    resource: META_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: META_ID.to_owned(),
                    extra: vec![],
                },
                stream_path: None,
                guess_stream: false,
            })),
        });
    });
}

fn dispatch(runtime: &Runtime<TestEnv, TestModel>, action: ActionMetaDetails) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::MetaDetails(action),
        });
    });
}

fn stored_item(runtime: &Runtime<TestEnv, TestModel>) -> LibraryItem {
    runtime
        .model()
        .unwrap()
        .ctx
        .library
        .items
        .get(META_ID)
        .unwrap()
        .to_owned()
}

#[test]
fn watched_falls_back_to_times_watched_when_marker_is_unknown() {
    let watched_once = LibraryItemState {
        times_watched: 1,
        ..Default::default()
    };
    assert!(create_library_item("series", watched_once.to_owned()).watched());
    assert!(create_library_item("movie", watched_once.to_owned()).watched());

    let marker_false = LibraryItemState {
        all_episodes_watched: Some(false),
        ..watched_once.to_owned()
    };
    assert!(
        !create_library_item("series", marker_false.to_owned()).watched(),
        "series use the marker once it is known",
    );
    assert!(
        create_library_item("movie", marker_false).watched(),
        "other types ignore the marker",
    );
}

#[test]
fn mark_as_watched_sets_series_marker_and_leaves_episodes() {
    let mut series = create_library_item("series", LibraryItemState::default());
    series.mark_as_watched::<TestEnv>(true);
    assert_eq!(series.state.all_episodes_watched, Some(true));
    assert_eq!(series.state.watched, None, "episodes bitfield is untouched");
    assert!(series.watched());

    series.mark_as_watched::<TestEnv>(false);
    assert_eq!(series.state.all_episodes_watched, Some(false));
    assert_eq!(series.state.times_watched, 0);
    assert!(!series.watched());

    let mut movie = create_library_item("movie", LibraryItemState::default());
    movie.mark_as_watched::<TestEnv>(true);
    assert_eq!(
        movie.state.all_episodes_watched, None,
        "movies don't get a marker"
    );
    assert!(movie.watched());
}

#[test]
fn update_all_episodes_watched_ignores_specials_and_unreleased() {
    let now = Utc::now();
    let mut series = create_library_item(
        "series",
        LibraryItemState {
            all_episodes_watched: Some(false),
            ..Default::default()
        },
    );

    assert!(!series.update_all_episodes_watched(
        &meta_item(),
        &watched_bitfield(&["tt123456:1:1"]),
        &now
    ));
    assert_eq!(series.state.all_episodes_watched, Some(false));

    assert!(series.update_all_episodes_watched(
        &meta_item(),
        &watched_bitfield(&["tt123456:1:1", "tt123456:1:2"]),
        &now
    ));
    assert_eq!(
        series.state.all_episodes_watched,
        Some(true),
        "the unwatched special and the unreleased episode don't count",
    );
}

#[test]
fn update_all_episodes_watched_only_flips_to_true() {
    let now = Utc::now();
    let mut series = create_library_item(
        "series",
        LibraryItemState {
            all_episodes_watched: Some(true),
            ..Default::default()
        },
    );
    assert!(!series.update_all_episodes_watched(&meta_item(), &watched_bitfield(&[]), &now));
    assert_eq!(series.state.all_episodes_watched, Some(true));
}

#[test]
fn update_all_episodes_watched_skips_empty_episode_list_and_movies() {
    let now = Utc::now();
    let empty_meta_item = MetaItem {
        videos: vec![],
        ..meta_item()
    };
    let mut series = create_library_item("series", LibraryItemState::default());
    assert!(!series.update_all_episodes_watched(&empty_meta_item, &watched_bitfield(&[]), &now));
    assert_eq!(series.state.all_episodes_watched, None);

    let mut movie = create_library_item("movie", LibraryItemState::default());
    assert!(!movie.update_all_episodes_watched(&meta_item(), &watched_bitfield(&[]), &now));
    assert_eq!(movie.state.all_episodes_watched, None);
}

#[test]
fn loading_details_computes_unknown_marker_from_episodes() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    // played one episode past the threshold (the #565 case)
    let library_item = create_library_item(
        "series",
        LibraryItemState {
            times_watched: 1,
            watched: Some(watched_bitfield(&["tt123456:1:1"]).into()),
            ..Default::default()
        },
    );
    assert!(library_item.watched(), "falls back to times_watched");

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        let library_item = stored_item(&runtime);
        assert_eq!(library_item.state.all_episodes_watched, Some(false));
        assert!(!library_item.watched());
    });
}

#[test]
fn loading_details_computes_unknown_marker_as_watched() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item(
        "series",
        LibraryItemState {
            watched: Some(watched_bitfield(&["tt123456:1:1", "tt123456:1:2"]).into()),
            ..Default::default()
        },
    );

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        let library_item = stored_item(&runtime);
        assert_eq!(library_item.state.all_episodes_watched, Some(true));
        assert!(library_item.watched());
    });
}

#[test]
fn marking_last_episode_watches_series_and_unmarking_keeps_it() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item(
        "series",
        LibraryItemState {
            all_episodes_watched: Some(false),
            ..Default::default()
        },
    );

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        dispatch(
            &runtime,
            ActionMetaDetails::MarkVideoAsWatched(create_video(1, 1), true),
        );
        assert_eq!(
            stored_item(&runtime).state.all_episodes_watched,
            Some(false)
        );

        dispatch(
            &runtime,
            ActionMetaDetails::MarkVideoAsWatched(create_video(1, 2), true),
        );
        assert_eq!(stored_item(&runtime).state.all_episodes_watched, Some(true));

        dispatch(
            &runtime,
            ActionMetaDetails::MarkVideoAsWatched(create_video(1, 2), false),
        );
        let library_item = stored_item(&runtime);
        assert_eq!(
            library_item.state.all_episodes_watched,
            Some(true),
            "unwatching an episode doesn't unwatch the series",
        );
        assert!(library_item.watched());
    });
}

#[test]
fn marking_season_watches_series() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item(
        "series",
        LibraryItemState {
            all_episodes_watched: Some(false),
            ..Default::default()
        },
    );

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        dispatch(&runtime, ActionMetaDetails::MarkSeasonAsWatched(1, true));
        assert_eq!(stored_item(&runtime).state.all_episodes_watched, Some(true));
    });
}

#[test]
fn marking_series_from_details_leaves_episodes() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let watched = watched_bitfield(&["tt123456:1:1"]);
    let library_item = create_library_item(
        "series",
        LibraryItemState {
            all_episodes_watched: Some(false),
            watched: Some(watched.to_owned().into()),
            ..Default::default()
        },
    );

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        dispatch(&runtime, ActionMetaDetails::MarkAsWatched(true));
        let library_item = stored_item(&runtime);
        assert_eq!(library_item.state.all_episodes_watched, Some(true));
        assert_eq!(library_item.state.watched, Some(watched.to_owned().into()));
        assert!(library_item.watched());

        dispatch(&runtime, ActionMetaDetails::MarkAsWatched(false));
        let library_item = stored_item(&runtime);
        assert_eq!(library_item.state.all_episodes_watched, Some(false));
        assert_eq!(library_item.state.watched, Some(watched.into()));
        assert!(!library_item.watched());
    });
}

#[test]
fn unmarking_series_with_all_episodes_watched_sticks() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let library_item = create_library_item(
        "series",
        LibraryItemState {
            times_watched: 1,
            all_episodes_watched: Some(true),
            watched: Some(watched_bitfield(&["tt123456:1:1", "tt123456:1:2"]).into()),
            ..Default::default()
        },
    );

    run_with_library_item(library_item, |runtime| {
        load_meta_details(&runtime);

        dispatch(&runtime, ActionMetaDetails::MarkAsWatched(false));
        let library_item = stored_item(&runtime);
        assert_eq!(library_item.state.all_episodes_watched, Some(false));
        assert!(!library_item.watched());
    });
}
