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
        profile::{Profile, Settings},
        resource::{
            MetaItem, MetaItemPreview, SeriesInfo, Stream, StreamBehaviorHints, StreamSource, Video,
        },
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

fn create_stream(binge_group: &str) -> Stream {
    Stream {
        source: StreamSource::Url {
            url: "https://source_url".parse().unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints {
            binge_group: Some(binge_group.to_owned()),
            ..Default::default()
        },
    }
}

fn create_video(season: u32, episode: u32) -> Video {
    Video {
        id: format!("tt123456:{season}:{episode}"),
        title: format!("video_{episode}"),
        released: None,
        overview: None,
        thumbnail: None,
        streams: vec![],
        series_info: Some(SeriesInfo { season, episode }),
        trailer_streams: vec![],
    }
}

#[test]
fn next_stream() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
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
                        videos: vec![create_video(1, 1), create_video(1, 2)],
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. }
                if url == "https://transport_url/stream/series/tt123456%3A1%3A2.json" =>
            {
                future::ok(Box::new(ResourceResponse::Streams {
                    streams: vec![create_stream("binge_group"), create_stream("binge_group_1")],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. }
                if url == "https://transport_url/stream/series/tt123456%3A1%3A3.json" =>
            {
                future::ok(Box::new(ResourceResponse::Streams {
                    streams: vec![create_stream("binge_group"), create_stream("binge_group_1")],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    settings: Settings {
                        binge_watching: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            player: Player::default(),
        },
        vec![],
        1000,
    );

    let stream = create_stream("binge_group");
    let meta_request = ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: META_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            id: "tt123456".to_owned(),
            extra: vec![],
        },
    };
    let stream_request = ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: STREAM_RESOURCE_NAME.to_owned(),
            r#type: "series".to_owned(),
            id: "tt123456:1:1".to_owned(),
            extra: vec![],
        },
    };

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::Player(Box::new(Selected {
                stream: stream.clone(),
                stream_request: Some(stream_request),
                meta_request: Some(meta_request),
                subtitles_path: None,
            }))),
        });
    });

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::TimeChanged {
                time: 50,
                duration: 100,
                device: "device".to_owned(),
            }),
        });
    });

    assert_eq!(
        runtime
            .model()
            .unwrap()
            .player
            .next_stream
            .as_ref()
            .unwrap()
            .behavior_hints
            .binge_group,
        stream.behavior_hints.binge_group,
        "next stream has same binge group"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::NextVideo),
        });
    });

    assert_eq!(
        runtime
            .model()
            .unwrap()
            .player
            .next_stream
            .as_ref()
            .unwrap()
            .behavior_hints
            .binge_group,
        stream.behavior_hints.binge_group,
        "next stream has same binge group"
    );

    assert_eq!(
        runtime
            .model()
            .unwrap()
            .player
            .library_item
            .as_ref()
            .unwrap()
            .state
            .time_offset,
        1,
        "library item time_offset was reset to 1 to keep it in CW as there's a next video"
    );
}
