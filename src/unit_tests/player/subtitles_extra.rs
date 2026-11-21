use std::any::Any;

use futures::future;
use serde_json::json;
use stremio_derive::Model;

use crate::{
    constants::SUBTITLES_RESOURCE_NAME,
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer},
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestResource, ResourcePath, ResourceResponse},
        resource::{Stream, StreamBehaviorHints, StreamSource},
    },
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn video_params_are_sent_to_subtitles_addons() {
    #[derive(Model, Default, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        if request.url.starts_with("https://subtitles.example/") {
            future::ok(
                Box::new(ResourceResponse::Subtitles { subtitles: vec![] }) as Box<dyn Any + Send>
            )
            .boxed_env()
        } else {
            default_fetch_handler(request)
        }
    });

    let mut model = TestModel::default();
    model.ctx.profile.addons = vec![Descriptor {
        transport_url: "https://subtitles.example/manifest.json".parse().unwrap(),
        manifest: Manifest {
            id: "subtitles".to_owned(),
            types: vec!["movie".to_owned()],
            resources: vec![ManifestResource::Short(SUBTITLES_RESOURCE_NAME.to_owned())],
            ..Default::default()
        },
        flags: Default::default(),
    }];
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    let dispatch = |action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                action,
                field: None,
            })
        });
    };
    let video_params_changed = |video_params| {
        dispatch(Action::Player(
            serde_json::from_value::<ActionPlayer>(json!({
                "action": "VideoParamsChanged",
                "args": { "videoParams": video_params },
            }))
            .unwrap(),
        ))
    };
    let last_requested_url = || REQUESTS.read().unwrap().last().unwrap().url.clone();

    dispatch(Action::Load(ActionLoad::Player(Box::new(Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: "https://media.example/film.mkv".parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
        stream_request: None,
        meta_request: None,
        subtitles_path: Some(ResourcePath::without_extra(
            SUBTITLES_RESOURCE_NAME,
            "movie",
            "tt1234567",
        )),
    }))));

    video_params_changed(json!({
        "hash": "8e245d9679d31e12",
        "size": 1632335363,
        "filename": "film.mkv",
    }));
    assert_eq!(
        last_requested_url(),
        "https://subtitles.example/subtitles/movie/tt1234567/filename=film.mkv&videoSize=1632335363&videoHash=8e245d9679d31e12.json"
    );

    video_params_changed(json!({
        "hash": "8e245d9679d31e12",
        "size": 1632335363,
        "filename": "film.mkv",
        "fpsMilli": 23976,
        "durationMs": 6123456,
        "embeddedSubtitleSignature": "v1:3:41c2",
    }));
    assert_eq!(
        last_requested_url(),
        "https://subtitles.example/subtitles/movie/tt1234567/embeddedSubtitleSignature=v1%3A3%3A41c2&durationMs=6123456&fpsMilli=23976&filename=film.mkv&videoSize=1632335363&videoHash=8e245d9679d31e12.json"
    );
}
