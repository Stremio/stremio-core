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
    types::addon::{Descriptor, Manifest, ResourcePath, ResourceRequest},
    unit_tests::{TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn removing_a_pending_metadata_addon_uses_the_cached_movie_metadata() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| future::pending().boxed_env());
    let addon = |name: &str| Descriptor {
        transport_url: format!("https://{name}.example/manifest.json")
            .parse()
            .unwrap(),
        manifest: Manifest {
            id: name.to_owned(),
            name: name.to_owned(),
            types: vec!["movie".to_owned()],
            resources: vec!["meta".into(), "stream".into()],
            ..Default::default()
        },
        flags: Default::default(),
    };
    let mut ctx = Ctx::default();
    ctx.profile.addons = vec![addon("preferred"), addon("fallback")];
    let mut details = MetaDetails::default();
    let update = |details: &mut MetaDetails, msg, ctx: &Ctx| {
        <MetaDetails as UpdateWithCtx<TestEnv>>::update(details, &msg, ctx)
    };
    let meta_path = ResourcePath::without_extra("meta", "movie", "tt1");
    update(
        &mut details,
        Msg::Action(Action::Load(ActionLoad::MetaDetails(Selected {
            meta_path: meta_path.clone(),
            stream_path: None,
            guess_stream: true,
        }))),
        &ctx,
    );
    let fallback_url = ctx.profile.addons[1].transport_url.clone();
    update(
        &mut details,
        Msg::Internal(Internal::ResourceRequestResult(
            ResourceRequest::new(fallback_url.clone(), meta_path),
            Box::new(Ok(serde_json::from_value(json!({
                "meta": {
                    "id": "tt1", "name": "Movie", "type": "movie",
                    "behaviorHints": {"defaultVideoId": "feature"}
                }
            }))
            .unwrap())),
        )),
        &ctx,
    );
    assert!(matches!(
        details.meta_items[0].content,
        Some(Loadable::Loading)
    ));
    assert!(details.streams.is_empty());

    ctx.profile.addons.remove(0);
    update(&mut details, Msg::Internal(Internal::ProfileChanged), &ctx);

    let stream_path = ResourcePath::without_extra("stream", "movie", "feature");
    assert_eq!(
        details.selected.unwrap().stream_path,
        Some(stream_path.clone())
    );
    assert_eq!(details.streams.len(), 1);
    assert_eq!(
        details.streams[0].request,
        ResourceRequest::new(fallback_url, stream_path)
    );
    assert_eq!(
        REQUESTS
            .read()
            .unwrap()
            .iter()
            .map(|request| request.url.as_str())
            .collect::<Vec<_>>(),
        [
            "https://preferred.example/meta/movie/tt1.json",
            "https://fallback.example/meta/movie/tt1.json",
            "https://fallback.example/stream/movie/feature.json",
        ]
    );
}
