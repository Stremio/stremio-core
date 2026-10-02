use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures::{channel::mpsc::Receiver, future};
use serde_json::{json, Value};
use stremio_derive::Model;
use url::Url;

use crate::{
    models::{
        common::Loadable,
        ctx::Ctx,
        streaming_server::{CastingStatus, PlaybackDevice, StreamingServer},
    },
    runtime::{
        msg::{Action, ActionCtx, ActionStreamingServer, CastingSubtitles, PlayOnDeviceArgs},
        EnvError, EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent,
    },
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER, REQUESTS},
};

#[derive(Model, Clone, Debug)]
#[model(TestEnv)]
struct CastModel {
    ctx: Ctx,
    streaming_server: StreamingServer,
}

fn runtime() -> (
    Runtime<TestEnv, CastModel>,
    Receiver<RuntimeEvent<TestEnv, CastModel>>,
) {
    let ctx = Ctx::default();
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| {
        future::err(EnvError::Other(
            "Initial discovery is supplied by the fixture".to_owned(),
        ))
        .boxed_env()
    });
    let (mut streaming_server, _) = StreamingServer::new::<TestEnv>(&ctx.profile);
    streaming_server.playback_devices = Loadable::Ready(
        [
            ("a", "chromecast"),
            ("b", "tv"),
            ("c", "chromecast"),
            ("vlc", "external"),
        ]
        .into_iter()
        .map(|(id, kind)| PlaybackDevice {
            id: id.to_owned(),
            name: id.to_owned(),
            r#type: kind.to_owned(),
        })
        .collect(),
    );
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        if request.method == "POST" && request.url.contains("/casting/") {
            future::ok(Box::new(json!({})) as Box<dyn Any + Send>).boxed_env()
        } else {
            default_fetch_handler(request)
        }
    });
    REQUESTS.write().unwrap().clear();
    Runtime::new(
        CastModel {
            ctx,
            streaming_server,
        },
        vec![],
        1000,
    )
}

fn dispatch(runtime: &Runtime<TestEnv, CastModel>, action: ActionStreamingServer) {
    runtime.dispatch(RuntimeAction {
        field: None,
        action: Action::StreamingServer(action),
    });
}

fn play(device: &str) -> PlayOnDeviceArgs {
    PlayOnDeviceArgs {
        device: device.to_owned(),
        source: "http://127.0.0.1:11470/movie.mp4".to_owned(),
        time: Some(12500),
        subtitles: Some(CastingSubtitles {
            subtitles_src: Some(Url::parse("https://example.com/en.srt").unwrap()),
            subtitles_delay: 500,
        }),
    }
}

fn requests() -> Vec<(String, Value)> {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .filter(|request| request.method == "POST" && request.url.contains("/casting/"))
        .map(|request| {
            (
                request.url.clone(),
                serde_json::from_str(&request.body).unwrap(),
            )
        })
        .collect()
}

#[test]
fn casting_initializes_source_before_subtitles_and_coalesces_off() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a")));
        let id = runtime
            .model()
            .unwrap()
            .streaming_server
            .casting
            .as_ref()
            .unwrap()
            .id;
        dispatch(
            &runtime,
            ActionStreamingServer::SetCastingSubtitles {
                id,
                subtitles: CastingSubtitles {
                    subtitles_src: Some(Url::parse("https://example.com/fr.srt").unwrap()),
                    subtitles_delay: -1000,
                },
            },
        );
        dispatch(
            &runtime,
            ActionStreamingServer::SetCastingSubtitles {
                id,
                subtitles: CastingSubtitles::default(),
            },
        );
    });
    assert_eq!(
        requests(),
        vec![
            (
                "http://127.0.0.1:11470/casting/a/player".to_owned(),
                json!({ "source": "http://127.0.0.1:11470/movie.mp4", "time": 12500 })
            ),
            (
                "http://127.0.0.1:11470/casting/a/player".to_owned(),
                json!({ "subtitlesSrc": "https://example.com/en.srt", "subtitlesDelay": 500 })
            ),
            (
                "http://127.0.0.1:11470/casting/a/player".to_owned(),
                json!({ "subtitlesSrc": null, "subtitlesDelay": 0 })
            ),
        ]
    );
    assert!(matches!(
        runtime
            .model()
            .unwrap()
            .streaming_server
            .casting
            .as_ref()
            .unwrap()
            .status,
        CastingStatus::Playing
    ));
}

#[test]
fn casting_stop_during_start_finishes_with_stop() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a")));
        dispatch(&runtime, ActionStreamingServer::StopCasting);
        dispatch(&runtime, ActionStreamingServer::StopCasting);
    });
    assert_eq!(requests().last().unwrap().1, json!({ "source": null }));
    assert_eq!(requests().len(), 3);
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
}

#[test]
fn casting_replacement_preserves_stop_and_discards_superseded_start() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a")));
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("b")));
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("c")));
    });
    let requests = requests();
    assert_eq!(requests.len(), 5);
    assert_eq!(
        requests[2],
        (
            "http://127.0.0.1:11470/casting/a/player".to_owned(),
            json!({ "source": null })
        )
    );
    assert!(requests[3].0.ends_with("/casting/c/player"));
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .streaming_server
            .casting
            .as_ref()
            .unwrap()
            .device,
        "c"
    );
}

#[test]
fn cancelling_replacement_waits_for_original_device_to_stop() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a")));
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("b")));
        dispatch(&runtime, ActionStreamingServer::StopCasting);
        let model = runtime.model().unwrap();
        let session = model.streaming_server.casting.as_ref().unwrap();
        assert_eq!(session.device, "a");
        assert!(matches!(session.status, CastingStatus::Stopping));
    });
    assert_eq!(requests().len(), 3);
    assert_eq!(requests().last().unwrap().1, json!({ "source": null }));
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
}

#[test]
fn failed_stop_blocks_replacement_and_can_be_retried() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    let fail_stop = Arc::new(AtomicBool::new(true));
    let fail = fail_stop.clone();
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request| {
        let body: Value = serde_json::from_str(&request.body).unwrap();
        let response = if body.get("source") == Some(&Value::Null) && fail.load(Ordering::SeqCst) {
            json!({ "error": "Device disconnected" })
        } else {
            json!({})
        };
        future::ok(Box::new(response) as Box<dyn Any + Send>).boxed_env()
    });
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a")));
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("b")));
    });
    {
        let model = runtime.model().unwrap();
        let session = model.streaming_server.casting.as_ref().unwrap();
        assert_eq!(session.device, "a");
        assert!(matches!(session.status, CastingStatus::Err(_)));
    }
    assert_eq!(requests().len(), 3);
    fail_stop.store(false, Ordering::SeqCst);
    TestEnv::run(|| dispatch(&runtime, ActionStreamingServer::StopCasting));
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
}

#[test]
fn failed_start_cleans_up_and_external_launches_do_not_create_cast_sessions() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        let body: Value = serde_json::from_str(&request.body).unwrap();
        let response = if body.get("source").is_some_and(Value::is_string) {
            json!("Receiver rejected the stream")
        } else {
            json!({})
        };
        future::ok(Box::new(response) as Box<dyn Any + Send>).boxed_env()
    });
    TestEnv::run(|| dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a"))));
    assert_eq!(requests().len(), 2);
    assert_eq!(requests().last().unwrap().1, json!({ "source": null }));
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
    TestEnv::run(|| dispatch(&runtime, ActionStreamingServer::PlayOnDevice(play("vlc"))));
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
    assert_eq!(requests().len(), 3);
}

#[test]
fn cast_controls_keep_the_original_endpoint_after_settings_and_discovery_change() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| dispatch(&runtime, ActionStreamingServer::CastToDevice(play("a"))));
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        if request.method == "GET" {
            future::err(EnvError::Other("New server unavailable".to_owned())).boxed_env()
        } else {
            future::ok(Box::new(json!({})) as Box<dyn Any + Send>).boxed_env()
        }
    });
    let mut settings = runtime.model().unwrap().ctx.profile.settings.clone();
    settings.streaming_server_url = Url::parse("http://127.0.0.1:21470/").unwrap();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(settings)),
        })
    });
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .streaming_server
            .selected
            .transport_url
            .as_str(),
        "http://127.0.0.1:21470/"
    );
    TestEnv::run(|| dispatch(&runtime, ActionStreamingServer::StopCasting));
    assert_eq!(
        requests().last().unwrap(),
        &(
            "http://127.0.0.1:11470/casting/a/player".to_owned(),
            json!({ "source": null })
        )
    );
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
}

#[test]
fn cast_actions_do_not_control_external_or_unknown_devices() {
    let _env = TestEnv::reset().unwrap();
    let (runtime, _rx) = runtime();
    TestEnv::run(|| {
        dispatch(&runtime, ActionStreamingServer::CastToDevice(play("vlc")));
        dispatch(
            &runtime,
            ActionStreamingServer::CastToDevice(play("missing")),
        );
        dispatch(
            &runtime,
            ActionStreamingServer::SetCastingSubtitles {
                id: 1,
                subtitles: CastingSubtitles::default(),
            },
        );
        dispatch(&runtime, ActionStreamingServer::StopCasting);
    });
    assert!(requests().is_empty());
    assert!(runtime.model().unwrap().streaming_server.casting.is_none());
}
