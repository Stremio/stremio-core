use crate::{
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionPlayer},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        profile::{Profile, Settings},
        resource::{Stream, StreamBehaviorHints, StreamSource},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};
use futures::future;
use std::any::Any;
use std::sync::{Arc, Mutex};
use stremio_derive::Model;
use url::Url;

#[test]
fn player_webhook_events() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let webhook_url = "https://webhook.site/test_webhook";
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_clone = requests.clone();

    let fetch_handler = move |request: Request| -> TryEnvFuture<Box<dyn Any + Send>> {
        if request.url == webhook_url {
            requests_clone.lock().unwrap().push(request);
            future::ok(Box::new(serde_json::Value::Null) as Box<dyn Any + Send>).boxed_env()
        } else {
            default_fetch_handler(request)
        }
    };

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    settings: Settings::default(),
                    ..Default::default()
                },
                ..Default::default()
            },
            player: Player {
                selected: Some(Selected {
                    stream: Stream {
                        source: StreamSource::Url {
                            url: "https://source_url".parse().unwrap(),
                        },
                        name: None,
                        description: None,
                        thumbnail: None,
                        subtitles: vec![],
                        behavior_hints: StreamBehaviorHints {
                            playback_webhook: Some(Url::parse(webhook_url).unwrap()),
                            ..Default::default()
                        },
                    },
                    stream_request: None,
                    meta_request: None,
                    subtitles_path: None,
                }),
                ..Default::default()
            },
        },
        vec![],
        1000,
    );

    // 1. Test PausedChanged -> playing (since loaded starts as false)
    assert!(!runtime.model().unwrap().player.loaded);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::PausedChanged { paused: false }),
        });
    });

    // Loaded should be set to true and a POST request sent to webhook
    assert!(runtime.model().unwrap().player.loaded);
    {
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].method, "POST");
        assert!(reqs[0].body.contains(r#""event":"playing""#));
    }

    // 2. Test PausedChanged -> paused
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::PausedChanged { paused: true }),
        });
    });
    {
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 2);
        assert!(reqs[1].body.contains(r#""event":"paused""#));
    }

    // 3. Test PausedChanged -> resumed
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::PausedChanged { paused: false }),
        });
    });
    {
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 3);
        assert!(reqs[2].body.contains(r#""event":"resumed""#));
    }

    // 4. Test Ended -> ended
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Player(ActionPlayer::Ended),
        });
    });
    {
        let reqs = requests.lock().unwrap();
        assert_eq!(reqs.len(), 4);
        assert!(reqs[3].body.contains(r#""event":"ended""#));
    }
}
