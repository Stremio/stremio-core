use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer},
        EnvError, EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture,
    },
    types::{
        addon::{
            Descriptor, Manifest, ManifestResource, ResourcePath, ResourceRequest, ResourceResponse,
        },
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, SeriesInfo, Stream, StreamSource, Video},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS},
};
use futures::{channel::mpsc::Receiver, future};
use std::any::Any;
use stremio_derive::Model;

#[derive(Model, Default, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    player: Player,
}

fn create_video(episode: u32) -> Video {
    Video {
        id: format!("tt123456:1:{episode}"),
        title: format!("S1E{episode}"),
        released: None,
        overview: None,
        thumbnail: None,
        streams: vec![],
        series_info: Some(SeriesInfo { season: 1, episode }),
        epg_info: None,
        trailer_streams: vec![],
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url == "https://transport_url/meta/series/tt123456.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: MetaItem {
                    preview: MetaItemPreview {
                        id: "tt123456".to_owned(),
                        r#type: "series".to_owned(),
                        ..Default::default()
                    },
                    videos: vec![create_video(1), create_video(2)],
                },
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        Request { url, .. } if url.starts_with("https://transport_url/stream/") => future::ok(
            Box::new(ResourceResponse::Streams { streams: vec![] }) as Box<dyn Any + Send>,
        )
        .boxed_env(),
        Request { url, .. } if url.starts_with("https://tracker/") => {
            future::err(EnvError::Fetch("ignored".to_owned())).boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

fn new_runtime() -> (
    Runtime<TestEnv, TestModel>,
    Receiver<RuntimeEvent<TestEnv, TestModel>>,
) {
    let tracker = Descriptor {
        manifest: Manifest {
            resources: vec![
                ManifestResource::Short("player".to_owned()),
                ManifestResource::Short("library".to_owned()),
            ],
            types: vec!["series".to_owned()],
            id_prefixes: Some(vec!["tt".to_owned()]),
            ..Default::default()
        },
        transport_url: "https://tracker/manifest.json".parse().unwrap(),
        flags: Default::default(),
    };
    let library_item = LibraryItem {
        id: "tt123456".into(),
        name: "Test Series".into(),
        r#type: "series".into(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: Default::default(),
        state: Default::default(),
        behavior_hints: Default::default(),
    };
    Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![tracker],
                    ..Default::default()
                },
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
    )
}

fn dispatch(runtime: &Runtime<TestEnv, TestModel>, action: Action) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action,
        })
    });
}

fn load(runtime: &Runtime<TestEnv, TestModel>) {
    let request = |resource: &str, id: &str| ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath::without_extra(resource, "series", id),
    };
    dispatch(
        runtime,
        Action::Load(ActionLoad::Player(Box::new(Selected {
            stream: Stream {
                source: StreamSource::Url {
                    url: "https://source_url".parse().unwrap(),
                },
                name: None,
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: Default::default(),
            },
            stream_request: Some(request(STREAM_RESOURCE_NAME, "tt123456:1:1")),
            meta_request: Some(request(META_RESOURCE_NAME, "tt123456")),
            subtitles_path: None,
        }))),
    );
}

fn player_action(runtime: &Runtime<TestEnv, TestModel>, action: ActionPlayer) {
    dispatch(runtime, Action::Player(action));
}

fn tracker_urls() -> Vec<String> {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .map(|request| request.url.to_owned())
        .filter(|url| url.starts_with("https://tracker/"))
        .collect()
}

#[test]
fn playback_sends_start_pause_and_stop() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = new_runtime();

    load(&runtime);
    player_action(
        &runtime,
        ActionPlayer::TimeChanged {
            time: 600_000,
            duration: 3_600_000,
            device: "test".to_owned(),
        },
    );
    player_action(&runtime, ActionPlayer::PausedChanged { paused: false });
    player_action(
        &runtime,
        ActionPlayer::Seek {
            time: 900_000,
            duration: 3_600_000,
            device: "test".to_owned(),
        },
    );
    player_action(&runtime, ActionPlayer::PausedChanged { paused: true });
    dispatch(&runtime, Action::Unload);

    let url = "https://tracker/player/series/tt123456%3A1%3A1";
    assert_eq!(
        tracker_urls(),
        vec![
            format!("{url}/action=start&currentTime=600000&duration=3600000.json"),
            format!("{url}/action=start&currentTime=900000&duration=3600000.json"),
            format!("{url}/action=pause&currentTime=900000&duration=3600000.json"),
            format!("{url}/action=stop&currentTime=900000&duration=3600000.json"),
        ],
    );
}

#[test]
fn ended_playback_sends_a_single_stop() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = new_runtime();

    load(&runtime);
    player_action(
        &runtime,
        ActionPlayer::TimeChanged {
            time: 3_590_000,
            duration: 3_600_000,
            device: "test".to_owned(),
        },
    );
    player_action(&runtime, ActionPlayer::PausedChanged { paused: false });
    player_action(&runtime, ActionPlayer::Ended);
    dispatch(&runtime, Action::Unload);

    let url = "https://tracker/player/series/tt123456%3A1%3A1";
    assert_eq!(
        tracker_urls(),
        vec![
            format!("{url}/action=start&currentTime=3590000&duration=3600000.json"),
            format!("{url}/action=stop&currentTime=3590000&duration=3600000.json"),
        ],
        "automatic watched is left to the tracker, no library event is sent"
    );
}

#[test]
fn mark_season_as_watched_sends_one_library_event_per_video() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = new_runtime();

    load(&runtime);
    player_action(&runtime, ActionPlayer::MarkSeasonAsWatched(1, true));

    let url = "https://tracker/library/series/tt123456/action=watched&videoId=tt123456%3A1%3A";
    assert_eq!(
        tracker_urls(),
        vec![format!("{url}1.json"), format!("{url}2.json")],
    );
}
