use std::{any::Any, marker::PhantomData};

use futures::future;
use serde_json::json;
use stremio_derive::Model;

use crate::{
    models::{
        common::Loadable,
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{ResourcePath, ResourceRequest, ResourceResponse},
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
        resource::{MetaItem, Stream, StreamSource},
    },
    unit_tests::{TestEnv, FETCH_HANDLER},
};

#[test]
fn rejected_episode_stream_keeps_the_saved_resume_position() {
    #[derive(Model, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        player: Player,
    }

    let _guard = TestEnv::reset().unwrap();
    let meta: MetaItem = serde_json::from_value(json!({
        "id": "tt1", "type": "series", "name": "Example Series",
        "videos": [
            {"id": "tt1:1:1", "season": 1, "episode": 1},
            {"id": "tt1:1:2", "season": 1, "episode": 2},
            {"id": "tt1:1:3", "season": 1, "episode": 3}
        ]
    }))
    .unwrap();
    let mut item = LibraryItem::from((&meta.preview, PhantomData::<TestEnv>));
    item.removed = false;
    item.temp = false;
    item.state.video_id = Some("tt1:1:1".to_owned());
    item.state.time_offset = 600_000;
    item.state.duration = 3_600_000;
    let saved_state = item.state.clone();
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request| {
        let response = match request.url.as_str() {
            "https://addon.example/meta/series/tt1.json" => {
                ResourceResponse::Meta { meta: meta.clone() }
            }
            "https://addon.example/stream/series/tt1%3A1%3A3.json" => {
                ResourceResponse::Streams { streams: vec![] }
            }
            _ => panic!("Unexpected request to {}", request.url),
        };
        future::ok(Box::new(response) as Box<dyn Any + Send>).boxed_env()
    });

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![],
                    ..Default::default()
                },
                library: LibraryBucket::new(None, vec![item]),
                ..Default::default()
            },
            player: Player::default(),
        },
        vec![],
        1000,
    );
    let request = |resource, id| {
        ResourceRequest::new(
            "https://addon.example/manifest.json".parse().unwrap(),
            ResourcePath::without_extra(resource, "series", id),
        )
    };
    let load = |source| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::Player(Box::new(Selected {
                    stream: Stream {
                        source,
                        name: None,
                        description: None,
                        thumbnail: None,
                        subtitles: vec![],
                        behavior_hints: Default::default(),
                    },
                    stream_request: Some(request("stream", "tt1:1:2")),
                    meta_request: Some(request("meta", "tt1")),
                    subtitles_path: None,
                }))),
            });
        });
    };

    load(StreamSource::Rar {
        urls: vec![],
        file_idx: None,
        file_must_include: vec![],
    });
    {
        let model = runtime.model().unwrap();
        assert!(matches!(model.player.stream, Some(Loadable::Err(_))));
        assert_eq!(model.player.next_video.as_ref().unwrap().id, "tt1:1:3");
        assert_eq!(model.ctx.library.items["tt1"].state, saved_state);
        assert_eq!(
            model.player.library_item.as_ref().unwrap().state,
            saved_state
        );
    }

    load(StreamSource::Url {
        url: "https://media.example/episode-2.mp4".parse().unwrap(),
    });
    let model = runtime.model().unwrap();
    assert!(matches!(model.player.stream, Some(Loadable::Ready(_))));
    assert_eq!(model.ctx.library.items["tt1"].state.time_offset, 1);
}
