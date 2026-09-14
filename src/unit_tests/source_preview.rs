use crate::{
    models::{
        common::{Loadable, ResourceError, ResourceLoadable},
        ctx::Ctx,
        meta_details::Selected,
        source_preview::{SourcePreview, SourceStatus},
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effect, EffectFuture, EnvError, EnvFutureExt, TryEnvFuture, UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest, ResourceResponse},
        resource::{MetaItem, Stream},
    },
    unit_tests::{Request, TestEnv, FETCH_HANDLER},
};
use futures::{executor::block_on, future};
use serde_json::json;
use std::any::Any;

fn request(resource: &str) -> ResourceRequest {
    ResourceRequest::new(
        "https://example.com/manifest.json".parse().unwrap(),
        ResourcePath::without_extra(resource, "movie", "tt1"),
    )
}

fn resource<T>(name: &str, content: Loadable<T, ResourceError>) -> ResourceLoadable<T> {
    ResourceLoadable {
        request: request(name),
        content: Some(content),
    }
}

fn stream() -> Stream {
    serde_json::from_value(json!({"url": "https://example.com/movie.mp4"})).unwrap()
}

fn preview() -> SourcePreview {
    let mut preview = SourcePreview::default();
    preview.selected = Some(Selected {
        meta_path: request("meta").path,
        stream_path: Some(request("stream").path),
        guess_stream: false,
    });
    preview.meta_items = vec![resource("meta", Loadable::Ready(MetaItem::default()))];
    preview
}

#[test]
fn positive_source_wins_over_error_and_pending() {
    let mut model = preview();
    model.streams = vec![
        resource("stream", Loadable::Loading),
        resource(
            "stream",
            Loadable::Err(ResourceError::Env(EnvError::Fetch("offline".into()))),
        ),
        resource("stream", Loadable::Ready(vec![stream()])),
    ];
    assert_eq!(model.status(), SourceStatus::SourcesFound);
}

#[test]
fn empty_and_failed_are_different() {
    let mut model = preview();
    model.streams = vec![resource(
        "stream",
        Loadable::Err(ResourceError::EmptyContent),
    )];
    assert_eq!(model.status(), SourceStatus::NoSources);
    model.streams.push(resource(
        "stream",
        Loadable::Err(ResourceError::Env(EnvError::Fetch("offline".into()))),
    ));
    assert_eq!(model.status(), SourceStatus::CheckFailed);
    model.streams.push(resource("stream", Loadable::Loading));
    assert_eq!(model.status(), SourceStatus::Checking);
}

#[test]
fn external_links_do_not_count_as_video_sources() {
    let mut model = preview();
    model.streams = vec![resource(
        "stream",
        Loadable::Ready(vec![serde_json::from_value(
            json!({"externalUrl":"https://example.com/watch"}),
        )
        .unwrap()]),
    )];
    assert_eq!(model.status(), SourceStatus::ExternalOnly);
}

#[test]
fn embedded_sources_match_details_precedence() {
    let mut model = preview();
    model.meta_streams = vec![resource("stream", Loadable::Ready(vec![stream()]))];
    model.streams = vec![resource("stream", Loadable::Loading)];
    assert_eq!(model.status(), SourceStatus::SourcesFound);
}

#[test]
fn pending_metadata_is_not_no_provider() {
    let mut model = preview();
    model.meta_items = vec![resource("meta", Loadable::Loading)];
    assert_eq!(model.status(), SourceStatus::Checking);
    model.meta_items.clear();
    assert_eq!(model.status(), SourceStatus::NoProvider);
}

#[test]
fn unresolved_video_is_not_unavailable() {
    let mut model = preview();
    model.selected.as_mut().unwrap().stream_path = None;
    assert_eq!(model.status(), SourceStatus::ChooseVideo);
}

fn context() -> Ctx {
    let mut ctx = Ctx::default();
    ctx.profile.addons = vec![serde_json::from_value(json!({
        "transportUrl":"https://example.com/manifest.json",
        "manifest":{"id":"test","version":"1.0.0","name":"Test","types":["movie"],"resources":["meta","stream"],"catalogs":[]}
    })).unwrap()];
    ctx
}

fn fetch(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    let response = if request.url.contains("/meta/") {
        serde_json::from_value::<ResourceResponse>(json!({"meta":{"id":"tt1","type":"movie","name":"Movie","behaviorHints":{"defaultVideoId":"video1"},"videos":[{"id":"video1","title":"Movie","streams":[{"url":"https://example.com/movie.mp4"}]}]}})).unwrap()
    } else {
        assert!(request.url.contains("/stream/movie/video1.json"));
        ResourceResponse::Streams { streams: vec![] }
    };
    future::ok(Box::new(response) as Box<dyn Any + Send>).boxed_env()
}

fn resolve(effect: Effect) -> Msg {
    match effect {
        Effect::Future(EffectFuture::Concurrent(future) | EffectFuture::Sequential(future)) => {
            block_on(future)
        }
        Effect::Msg(message) => *message,
    }
}

#[test]
fn checks_resolved_video_without_library_effects() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch);
    let ctx = context();
    let mut model = SourcePreview::default();
    let mut work: Vec<_> = <SourcePreview as UpdateWithCtx<TestEnv>>::update(
        &mut model,
        &Msg::Action(Action::Load(ActionLoad::SourcePreview("tt1".into()))),
        &ctx,
    )
    .into_iter()
    .collect();
    assert_eq!(model.status(), SourceStatus::Checking);
    while let Some(effect) = work.pop() {
        let message = resolve(effect);
        assert!(matches!(
            message,
            Msg::Internal(Internal::SourcePreviewResult(..))
        ));
        work.extend(<SourcePreview as UpdateWithCtx<TestEnv>>::update(
            &mut model, &message, &ctx,
        ));
    }
    assert_eq!(model.status(), SourceStatus::SourcesFound);
    assert_eq!(model.selected.unwrap().stream_path.unwrap().id, "video1");
}

#[test]
fn late_responses_cannot_restore_unloaded_or_reconfigured_checks() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch);
    for invalidation in [
        Msg::Action(Action::Unload),
        Msg::Internal(Internal::ProfileChanged),
    ] {
        let ctx = context();
        let mut model = SourcePreview::default();
        let effect = <SourcePreview as UpdateWithCtx<TestEnv>>::update(
            &mut model,
            &Msg::Action(Action::Load(ActionLoad::SourcePreview("tt1".into()))),
            &ctx,
        )
        .into_iter()
        .next()
        .unwrap();
        let old_response = resolve(effect);
        <SourcePreview as UpdateWithCtx<TestEnv>>::update(&mut model, &invalidation, &ctx);
        <SourcePreview as UpdateWithCtx<TestEnv>>::update(&mut model, &old_response, &ctx);
        assert_eq!(model.status(), SourceStatus::Unknown);
        assert!(model.meta_items.is_empty());
    }
}
