use futures::future;
use serde_json::json;

use crate::{
    models::{
        common::Loadable,
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        EnvFutureExt, UpdateWithCtx,
    },
    types::{
        addon::{Descriptor, ResourcePath, ResourceRequest},
        profile::Profile,
    },
    unit_tests::{TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn details_starts_stream_requests_without_waiting_for_lower_priority_metadata() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| future::pending().boxed_env());
    let addons = ["preferred", "fallback", "slow"]
        .into_iter()
        .map(|name| {
            serde_json::from_value::<Descriptor>(json!({
                "transportUrl": format!("https://{name}.example/manifest.json"),
                "manifest": {
                    "id": name, "name": name, "version": "1.0.0",
                    "types": ["movie"], "resources": ["meta", "stream"], "catalogs": []
                }
            }))
            .unwrap()
        })
        .collect();
    let ctx = Ctx {
        profile: Profile {
            addons,
            ..Default::default()
        },
        ..Default::default()
    };
    let meta_path = ResourcePath::without_extra("meta", "movie", "tt1");
    let result = |index: usize, video_id: &str| {
        Msg::Internal(Internal::ResourceRequestResult(
            ResourceRequest::new(
                ctx.profile.addons[index].transport_url.clone(),
                meta_path.clone(),
            ),
            Box::new(Ok(serde_json::from_value(json!({
                "meta": {
                    "id": "tt1", "name": "Movie", "type": "movie",
                    "behaviorHints": { "defaultVideoId": video_id }
                }
            }))
            .unwrap())),
        ))
    };
    let mut details = MetaDetails::default();
    let update = |details: &mut MetaDetails, msg| {
        <MetaDetails as UpdateWithCtx<TestEnv>>::update(details, &msg, &ctx)
    };
    update(
        &mut details,
        Msg::Action(Action::Load(ActionLoad::MetaDetails(Selected {
            meta_path: meta_path.clone(),
            stream_path: None,
            guess_stream: true,
        }))),
    );

    update(&mut details, result(1, "fallback-video"));
    assert!(details.selected.as_ref().unwrap().stream_path.is_none());
    assert!(details.streams.is_empty());

    update(&mut details, result(0, "preferred-video"));
    assert!(matches!(
        details.meta_items[2].content,
        Some(Loadable::Loading)
    ));
    assert_eq!(
        details.selected.unwrap().stream_path,
        Some(ResourcePath::without_extra(
            "stream",
            "movie",
            "preferred-video"
        ))
    );
    let requests = REQUESTS.read().unwrap();
    let stream_requests: Vec<_> = requests
        .iter()
        .filter(|request| request.url.contains("/stream/"))
        .map(|request| request.url.as_str())
        .collect();
    assert_eq!(
        stream_requests,
        [
            "https://preferred.example/stream/movie/preferred-video.json",
            "https://fallback.example/stream/movie/preferred-video.json",
            "https://slow.example/stream/movie/preferred-video.json",
        ]
    );
}
