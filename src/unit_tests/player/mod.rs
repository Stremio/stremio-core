mod addon_events;
mod mark_video_as_watched;
mod next_stream;
mod stream_state;
mod subtitle_addon_change;

use crate::{
    constants::SUBTITLES_RESOURCE_NAME,
    models::{
        common::Loadable,
        ctx::Ctx,
        player::{Player, Selected, VideoParams},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer},
        EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestResource, ResourcePath, ResourceResponse},
        resource::{Stream, StreamBehaviorHints, StreamSource, Subtitles},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, EVENTS, FETCH_HANDLER, REQUESTS},
};
use futures::future;
use std::{
    any::Any,
    sync::{Arc, RwLock},
};
use stremio_derive::Model;

#[test]
fn identical_player_loads_are_observable() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel::default(), vec![], 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    let selected = Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: "https://source_url".parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
        stream_request: None,
        meta_request: None,
        subtitles_path: None,
    };

    TestEnv::run_with_runtime(rx, runtime.clone(), move || {
        let runtime = runtime.read().unwrap();
        for _ in 0..2 {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::Player(Box::new(selected.clone()))),
            });
        }
    });

    let events = EVENTS.read().unwrap();
    let new_states = events
        .iter()
        .filter_map(|event| {
            match event
                .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
                .unwrap()
            {
                RuntimeEvent::NewState(fields, _) => Some(fields),
                RuntimeEvent::CoreEvent(_) => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(new_states.len(), 2);
    for fields in new_states {
        assert_eq!(fields.as_slice(), [TestModelField::Player]);
    }
}

#[test]
fn subtitles_are_requested_when_playback_loads_without_video_params() {
    check_subtitle_requests(false);
}

#[test]
fn subtitles_are_not_requested_by_id_when_video_params_arrive_first() {
    check_subtitle_requests(true);
}

fn check_subtitle_requests(params_arrive_before_playback: bool) {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let subtitle = Subtitles {
        id: "english".to_owned(),
        lang: "eng".to_owned(),
        url: "https://addon.example/english.srt".parse().unwrap(),
        ..Default::default()
    };
    let response = ResourceResponse::Subtitles {
        subtitles: vec![subtitle.clone()],
    };
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request: Request| {
        if request.method == "GET" && request.url.starts_with("https://addon.example/subtitles/") {
            future::ok(Box::new(response.clone()) as Box<dyn Any + Send>).boxed_env()
        } else {
            default_fetch_handler(request)
        }
    });

    let mut model = TestModel::default();
    model.ctx.profile.addons = vec![Descriptor {
        manifest: Manifest {
            id: "test-subtitles".to_owned(),
            types: vec!["movie".to_owned()],
            resources: vec![ManifestResource::Short(SUBTITLES_RESOURCE_NAME.to_owned())],
            id_prefixes: Some(vec![]),
            ..Default::default()
        },
        transport_url: "https://addon.example/manifest.json".parse().unwrap(),
        flags: Default::default(),
    }];
    let (runtime, _events) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    let selected = Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: "https://media.example/film.mp4".parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
        stream_request: None,
        meta_request: None,
        subtitles_path: Some(ResourcePath {
            resource: SUBTITLES_RESOURCE_NAME.to_owned(),
            r#type: "movie".to_owned(),
            id: "tt1234567".to_owned(),
            extra: vec![],
        }),
    };
    let dispatch = |action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action,
            })
        });
    };

    dispatch(Action::Load(ActionLoad::Player(Box::new(selected))));
    assert!(REQUESTS.read().unwrap().is_empty());

    let params_changed = Action::Player(ActionPlayer::VideoParamsChanged {
        video_params: Some(VideoParams {
            hash: Some("abcd1234".to_owned()),
            size: Some(1024),
            filename: Some("film.mp4".to_owned()),
        }),
    });
    let id_only_url = "https://addon.example/subtitles/movie/tt1234567.json";
    let enriched_url = "https://addon.example/subtitles/movie/tt1234567/filename=film.mp4&videoSize=1024&videoHash=abcd1234.json";

    let requested_urls = || {
        REQUESTS
            .read()
            .unwrap()
            .iter()
            .map(|request| request.url.to_owned())
            .collect::<Vec<_>>()
    };

    if params_arrive_before_playback {
        dispatch(params_changed.clone());
    }
    dispatch(Action::Player(ActionPlayer::PausedChanged {
        paused: false,
    }));
    assert_eq!(
        requested_urls(),
        [if params_arrive_before_playback {
            enriched_url
        } else {
            id_only_url
        }]
    );
    assert_eq!(
        runtime.model().unwrap().player.subtitles[0].content,
        Some(Loadable::Ready(vec![subtitle]))
    );

    dispatch(params_changed);
    let expected = if params_arrive_before_playback {
        vec![enriched_url]
    } else {
        vec![id_only_url, enriched_url]
    };
    assert_eq!(requested_urls(), expected);
}
