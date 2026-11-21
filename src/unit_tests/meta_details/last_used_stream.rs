use std::any::Any;

use futures::future;
use serde_json::{from_value, json};
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{
        addon::{Descriptor, Manifest, ResourcePath, ResourceResponse},
        resource::{MetaItem, Stream},
        streams::{StreamsItem, StreamsItemKey},
    },
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER},
};

#[test]
fn last_used_stream_prefers_exact_match_across_lists_from_the_same_addon() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        details: MetaDetails,
    }

    let _guard = TestEnv::reset().unwrap();
    let base: Url = "https://addon.example/manifest.json".parse().unwrap();
    let id = "tt123456";
    let video_id = "tt123456:1:1";
    let last_played: Stream = from_value(json!({
        "url": "https://media.example/last.mp4",
        "behaviorHints": { "bingeGroup": "series" },
    }))
    .unwrap();
    let meta: MetaItem = from_value(json!({
        "id": id,
        "type": "series",
        "name": "Example Series",
        "videos": [{
            "id": video_id,
            "season": 1,
            "episode": 1,
            "streams": [{
                "url": "https://media.example/embedded.mp4",
                "behaviorHints": { "bingeGroup": "series" },
            }],
        }],
    }))
    .unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new({
        let last_played = last_played.clone();
        move |request| {
            let result = match request.url.as_str() {
                "https://addon.example/meta/series/tt123456.json" => {
                    ResourceResponse::Meta { meta: meta.clone() }
                }
                "https://addon.example/stream/series/tt123456%3A1%3A1.json" => {
                    ResourceResponse::Streams {
                        streams: vec![last_played.clone()],
                    }
                }
                _ => return default_fetch_handler(request),
            };
            future::ok(Box::new(result) as Box<dyn Any + Send>).boxed_env()
        }
    });
    let mut model = TestModel::default();
    model.ctx.profile.addons = vec![Descriptor {
        transport_url: base.clone(),
        manifest: Manifest {
            types: vec!["series".to_owned()],
            resources: vec![META_RESOURCE_NAME.into(), STREAM_RESOURCE_NAME.into()],
            ..Default::default()
        },
        flags: Default::default(),
    }];
    model.ctx.streams.items.insert(
        StreamsItemKey {
            meta_id: id.to_owned(),
            video_id: video_id.to_owned(),
        },
        StreamsItem {
            stream: last_played.clone(),
            r#type: "series".to_owned(),
            meta_id: id.to_owned(),
            video_id: video_id.to_owned(),
            meta_transport_url: base.clone(),
            stream_transport_url: base,
            state: None,
            mtime: Default::default(),
        },
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: ResourcePath::without_extra(META_RESOURCE_NAME, "series", id),
                stream_path: Some(ResourcePath::without_extra(
                    STREAM_RESOURCE_NAME,
                    "series",
                    video_id,
                )),
                guess_stream: false,
            })),
        });
    });

    let model = runtime.model().unwrap();
    let stream = model
        .details
        .last_used_stream
        .as_ref()
        .and_then(|result| result.content.as_ref()?.ready()?.as_ref())
        .unwrap();
    assert_eq!(stream, &last_played);
}
