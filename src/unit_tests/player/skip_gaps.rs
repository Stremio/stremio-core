use std::marker::PhantomData;

use chrono::Duration;
use futures::future;

use crate::{
    constants::STREAM_RESOURCE_NAME,
    models::{
        common::Loadable,
        ctx::{Ctx, CtxError},
        player::{AnalyticsContext, Player, Selected, VideoParams},
    },
    runtime::{
        msg::{Action, ActionPlayer, Internal, Msg},
        Env, EnvError, EnvFutureExt, UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest},
        api::{SkipGaps, SkipGapsRequest, SkipGapsResponse},
        library::LibraryItem,
        profile::{Auth, AuthKey, User},
        resource::{MetaItemPreview, SeriesInfo, Stream, StreamBehaviorHints, StreamSource},
    },
    unit_tests::{TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn delayed_skip_gaps_results_do_not_replace_the_current_episode() {
    let _guard = TestEnv::reset().unwrap();
    let request = |episode| SkipGapsRequest {
        auth_key: AuthKey("test-key".to_owned()),
        os_hash: format!("file-{episode}"),
        item_id: "tt123456".to_owned(),
        series_info: SeriesInfo { season: 1, episode },
        stream_name_hash: "stream".to_owned(),
        duration: 3_600_000,
    };
    let previous = request(1);
    let current = request(2);
    let duration = current.duration;
    let result = |outro| SkipGapsResponse {
        accuracy: "hash".to_owned(),
        gaps: [(
            duration,
            SkipGaps {
                seek_history: vec![],
                outro: Some(outro),
            },
        )]
        .into(),
    };
    let mut player = Player {
        skip_gaps: Some((current.clone(), Loadable::Loading)),
        library_item: Some(LibraryItem::from((
            &MetaItemPreview::default(),
            PhantomData::<TestEnv>,
        ))),
        ..Default::default()
    };
    let ctx = Ctx::default();
    let stale_result = Msg::Internal(Internal::SkipGapsResult(previous, Ok(result(3_200_000))));

    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &stale_result, &ctx);
    assert_eq!(player.skip_gaps, Some((current.clone(), Loadable::Loading)));
    assert!(player.intro_outro.is_none());

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Internal(Internal::SkipGapsResult(
            current.clone(),
            Ok(result(3_400_000)),
        )),
        &ctx,
    );
    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &stale_result, &ctx);
    assert_eq!(
        player.skip_gaps,
        Some((current, Loadable::Ready(result(3_400_000))))
    );
    assert_eq!(
        player
            .intro_outro
            .as_ref()
            .and_then(|timings| timings.outro),
        Some(3_400_000)
    );
}

#[test]
fn skip_gaps_are_requested_only_when_the_request_changes() {
    let _guard = TestEnv::reset().unwrap();
    *FETCH_HANDLER.write().unwrap() = Box::new(|_| future::pending().boxed_env());
    let mut ctx = Ctx::default();
    ctx.profile.auth = Some(Auth {
        key: AuthKey("test-key".to_owned()),
        user: User {
            premium_expire: Some(TestEnv::now() + Duration::days(1)),
            ..User::default()
        },
    });
    let video_params = |hash: &str| VideoParams {
        hash: Some(hash.to_owned()),
        size: None,
        filename: None,
    };
    let mut player = Player {
        selected: Some(Selected {
            stream: Stream {
                source: StreamSource::Url {
                    url: "https://source_url".parse().unwrap(),
                },
                name: Some("stream".to_owned()),
                description: None,
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: StreamBehaviorHints::default(),
            },
            stream_request: Some(ResourceRequest {
                base: "https://addon/manifest.json".parse().unwrap(),
                path: ResourcePath {
                    resource: STREAM_RESOURCE_NAME.to_owned(),
                    r#type: "series".to_owned(),
                    id: "tt123456:1:1".to_owned(),
                    extra: vec![],
                },
            }),
            meta_request: None,
            subtitles_path: None,
        }),
        video_params: Some(video_params("file-1")),
        series_info: Some(SeriesInfo {
            season: 1,
            episode: 1,
        }),
        library_item: Some(LibraryItem::from((
            &MetaItemPreview {
                id: "tt123456".to_owned(),
                r#type: "series".to_owned(),
                ..MetaItemPreview::default()
            },
            PhantomData::<TestEnv>,
        ))),
        analytics_context: Some(AnalyticsContext::default()),
        ..Default::default()
    };
    let update = |player: &mut Player, msg: Msg| {
        <Player as UpdateWithCtx<TestEnv>>::update(player, &msg, &ctx);
        REQUESTS
            .read()
            .unwrap()
            .iter()
            .filter(|request| request.url.ends_with("/getSkipGaps"))
            .count()
    };
    let time_changed = |time| {
        Msg::Action(Action::Player(ActionPlayer::TimeChanged {
            time,
            duration: 3_600_000,
            device: "device".to_owned(),
        }))
    };

    assert_eq!(update(&mut player, time_changed(1_000)), 1);
    let (request, _) = player.skip_gaps.clone().unwrap();
    assert_eq!(request.os_hash, "file-1");
    assert_eq!(request.duration, 3_600_000);
    assert_eq!(update(&mut player, time_changed(2_000)), 1);

    let error = CtxError::Env(EnvError::Fetch("offline".to_owned()));
    update(
        &mut player,
        Msg::Internal(Internal::SkipGapsResult(request.clone(), Err(error))),
    );
    assert_eq!(update(&mut player, time_changed(3_000)), 1);
    assert!(matches!(player.skip_gaps, Some((_, Loadable::Err(_)))));

    let video_params_changed = Msg::Action(Action::Player(ActionPlayer::VideoParamsChanged {
        video_params: Some(video_params("file-2")),
    }));
    assert_eq!(update(&mut player, video_params_changed), 2);
    assert_eq!(
        player.skip_gaps,
        Some((
            SkipGapsRequest {
                os_hash: "file-2".to_owned(),
                ..request
            },
            Loadable::Loading
        ))
    );
}
