use std::any::Any;

use futures::future;
use stremio_derive::Model;

use crate::{
    constants::SUBTITLES_RESOURCE_NAME,
    models::{
        common::Loadable,
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestResource, ResourcePath, ResourceResponse},
        resource::{Stream, StreamBehaviorHints, StreamSource, Subtitles},
    },
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn changing_addons_refreshes_subtitles_for_the_current_video() {
    #[derive(Model, Default, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _guard = TestEnv::reset().unwrap();
    let addon = |id: &str| Descriptor {
        transport_url: format!("https://{id}.example/manifest.json")
            .parse()
            .unwrap(),
        manifest: Manifest {
            id: id.to_owned(),
            types: vec!["movie".to_owned()],
            resources: vec![ManifestResource::Short(SUBTITLES_RESOURCE_NAME.to_owned())],
            ..Default::default()
        },
        flags: Default::default(),
    };
    let first = addon("first");
    let second = addon("second");
    const FIRST_URL: &str =
        "https://first.example/subtitles/movie/tt1234567/filename=film.mp4.json";
    const SECOND_URL: &str =
        "https://second.example/subtitles/movie/tt1234567/filename=film.mp4.json";

    let subtitle = Subtitles {
        id: "english".to_owned(),
        lang: "eng".to_owned(),
        url: "https://captions.example/english.srt".parse().unwrap(),
        ..Default::default()
    };
    *FETCH_HANDLER.write().unwrap() = Box::new({
        let subtitle = subtitle.clone();
        move |request| match request.url.as_str() {
            FIRST_URL | SECOND_URL => future::ok(Box::new(ResourceResponse::Subtitles {
                subtitles: vec![subtitle.clone()],
            }) as Box<dyn Any + Send>)
            .boxed_env(),
            _ => default_fetch_handler(request),
        }
    });

    let mut model = TestModel::default();
    model.ctx.profile.addons = vec![first.clone()];
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    let dispatch = |action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                action,
                field: None,
            })
        });
    };
    let requested_urls = || {
        REQUESTS
            .read()
            .unwrap()
            .iter()
            .map(|request| request.url.clone())
            .collect::<Vec<_>>()
    };

    dispatch(Action::Load(ActionLoad::Player(Box::new(Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: "https://media.example/film.mp4".parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints {
                filename: Some("film.mp4".to_owned()),
                ..Default::default()
            },
        },
        stream_request: None,
        meta_request: None,
        subtitles_path: Some(ResourcePath::without_extra(
            SUBTITLES_RESOURCE_NAME,
            "movie",
            "tt1234567",
        )),
    }))));
    assert_eq!(requested_urls(), [FIRST_URL]);

    dispatch(Action::Ctx(ActionCtx::InstallAddon(second.clone())));
    assert_eq!(requested_urls(), [FIRST_URL, SECOND_URL]);
    {
        let model = runtime.model().unwrap();
        assert_eq!(model.player.subtitles.len(), 2);
        assert_eq!(model.player.subtitles[1].request.base, second.transport_url);
        assert_eq!(
            model.player.subtitles[1].content,
            Some(Loadable::Ready(vec![subtitle]))
        );
    }

    dispatch(Action::Ctx(ActionCtx::UninstallAddon(second)));
    assert_eq!(requested_urls(), [FIRST_URL, SECOND_URL]);
    let model = runtime.model().unwrap();
    assert_eq!(model.player.subtitles.len(), 1);
    assert_eq!(model.player.subtitles[0].request.base, first.transport_url);
}
