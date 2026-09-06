use chrono::{Duration, TimeZone, Utc};
use futures::future;
use serde_json::json;

use crate::{
    models::{
        common::{Loadable, ResourceLoadable},
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionMetaDetails, Internal, Msg},
        EnvError, EnvFutureExt, UpdateWithCtx,
    },
    types::{
        addon::{Descriptor, ResourcePath, ResourceRequest, ResourceResponse},
        profile::Profile,
        resource::MetaItem,
    },
    unit_tests::{TestEnv, FETCH_HANDLER, NOW},
};

fn keep_requests_pending() {
    *FETCH_HANDLER.write().unwrap() = Box::new(|request| {
        assert!(request.url.starts_with("https://addon/"));
        future::pending().boxed_env()
    });
}

fn fixture(r#type: &str, explicit_live: bool) -> (MetaDetails, Ctx, ResourceRequest) {
    let addon: Descriptor = serde_json::from_value(json!({
        "transportUrl": "https://addon/manifest.json",
        "manifest": {
            "id": "live", "name": "Live", "version": "1.0.0",
            "types": [r#type], "resources": ["meta", "stream"], "catalogs": []
        }
    }))
    .unwrap();
    let request = ResourceRequest {
        base: addon.transport_url.clone(),
        path: ResourcePath {
            resource: "meta".into(),
            r#type: r#type.into(),
            id: "channel".into(),
            extra: vec![],
        },
    };
    let meta: MetaItem = serde_json::from_value(json!({
        "id": "channel", "type": r#type, "name": "Channel",
        "behaviorHints": { "isLive": explicit_live },
        "videos": [{
            "id": "programme-a", "title": "Programme A",
            "startTime": "2026-09-06T12:00:00Z", "endTime": "2026-09-06T13:00:00Z"
        }]
    }))
    .unwrap();
    let details = MetaDetails {
        meta_items: vec![ResourceLoadable {
            request: request.clone(),
            content: Some(Loadable::Ready(meta)),
        }],
        ..Default::default()
    };
    let ctx = Ctx {
        profile: Profile {
            addons: vec![addon],
            ..Default::default()
        },
        ..Default::default()
    };
    (details, ctx, request)
}

#[test]
fn live_channel_details_select_the_channel_stream_with_a_schedule() {
    let _guard = TestEnv::reset().unwrap();
    keep_requests_pending();
    for (r#type, explicit_live) in [("tv", false), ("channel", true)] {
        let (mut details, ctx, request) = fixture(r#type, explicit_live);
        <MetaDetails as UpdateWithCtx<TestEnv>>::update(
            &mut details,
            &Msg::Action(Action::Load(ActionLoad::MetaDetails(Selected {
                meta_path: request.path,
                stream_path: None,
                guess_stream: true,
            }))),
            &ctx,
        );
        assert_eq!(
            details
                .selected
                .as_ref()
                .unwrap()
                .stream_path
                .as_ref()
                .unwrap()
                .id,
            "channel"
        );
        assert_eq!(details.streams.len(), 1);
        assert_eq!(details.streams[0].request.path.id, "channel");
    }
}

#[test]
fn live_channel_details_refresh_in_background_and_retry_without_resetting_streams() {
    let _guard = TestEnv::reset().unwrap();
    keep_requests_pending();
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 9, 6, 12, 30, 0).unwrap();
    let (mut details, ctx, request) = fixture("tv", false);
    let update = |details: &mut MetaDetails, msg: Msg| {
        <MetaDetails as UpdateWithCtx<TestEnv>>::update(details, &msg, &ctx)
    };
    update(
        &mut details,
        Msg::Action(Action::Load(ActionLoad::MetaDetails(Selected {
            meta_path: request.path.clone(),
            stream_path: None,
            guess_stream: true,
        }))),
    );
    let stream_request = details.streams[0].request.clone();
    update(
        &mut details,
        Msg::Internal(Internal::ResourceRequestResult(
            stream_request,
            Box::new(Ok(serde_json::from_value(
                json!({"streams": [{"url": "https://channel/live.m3u8"}]}),
            )
            .unwrap())),
        )),
    );
    let displayed = details.meta_items.clone();
    let streams = details.streams.clone();
    let selected = details.selected.clone();
    let refresh = || Msg::Action(Action::MetaDetails(ActionMetaDetails::RefreshLive));
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 0);
    *NOW.write().unwrap() += Duration::minutes(14);
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 0);
    *NOW.write().unwrap() += Duration::minutes(1);
    let effects = update(&mut details, refresh());
    assert!(!effects.has_changed);
    assert_eq!(effects.into_iter().count(), 1);
    assert_eq!(details.meta_items, displayed);
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 0);

    update(
        &mut details,
        Msg::Internal(Internal::ResourceRequestResult(
            request.clone(),
            Box::new(Err(EnvError::Fetch("unavailable".into()))),
        )),
    );
    assert_eq!(details.meta_items, displayed);
    *NOW.write().unwrap() += Duration::seconds(30);
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 0);
    *NOW.write().unwrap() += Duration::seconds(30);
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 1);
    let mut updated = displayed[0]
        .content
        .as_ref()
        .unwrap()
        .ready()
        .unwrap()
        .clone();
    updated.videos[0].title = "Updated programme".into();
    updated.videos[0].epg_info.as_mut().unwrap().end_time =
        Utc.with_ymd_and_hms(2026, 9, 6, 12, 47, 0).unwrap();
    update(
        &mut details,
        Msg::Internal(Internal::ResourceRequestResult(
            request.clone(),
            Box::new(Ok(ResourceResponse::Meta {
                meta: updated.clone(),
            })),
        )),
    );
    assert_eq!(
        details.meta_items[0].content,
        Some(Loadable::Ready(updated))
    );
    assert_eq!(details.selected, selected);
    assert_eq!(details.streams, streams);

    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 9, 6, 12, 48, 0).unwrap();
    assert_eq!(update(&mut details, refresh()).into_iter().count(), 1);
    update(&mut details, Msg::Action(Action::Unload));
    assert!(details.live_schedule_refresh.is_none());

    let (mut movie, _, _) = fixture("movie", false);
    assert_eq!(update(&mut movie, refresh()).into_iter().count(), 0);
    assert!(movie.live_schedule_refresh.is_none());
}
