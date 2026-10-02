use chrono::{DateTime, Utc};

use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer, Internal, Msg},
        UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest},
        player::VideoScale,
        resource::{Stream, StreamBehaviorHints, StreamSource},
        streams::{StreamItemState, StreamsItem, StreamsItemKey},
    },
    unit_tests::TestEnv,
};

fn stream(source_url: &str, binge_group: Option<&str>) -> Stream {
    Stream {
        source: StreamSource::Url {
            url: source_url.parse().unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints {
            binge_group: binge_group.map(str::to_owned),
            ..Default::default()
        },
    }
}

fn selected(source_url: &str) -> Selected {
    Selected {
        stream: stream(source_url, None),
        stream_request: None,
        meta_request: None,
        subtitles_path: None,
    }
}

fn request(resource: &str, id: &str) -> ResourceRequest {
    ResourceRequest {
        base: "https://transport_url/manifest.json".parse().unwrap(),
        path: ResourcePath {
            resource: resource.to_owned(),
            r#type: "series".to_owned(),
            id: id.to_owned(),
            extra: vec![],
        },
    }
}

#[test]
fn video_scale_is_scoped_to_player_session() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let mut player = Player::default();
    let ctx = Ctx::default();
    let scale_msg = Msg::Action(Action::Player(ActionPlayer::VideoScaleChanged {
        video_scale: VideoScale::Cover,
    }));

    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &scale_msg, &ctx);
    assert!(!effects.has_changed);
    assert!(effects.is_empty());
    assert_eq!(player.video_scale, None);

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Load(ActionLoad::Player(Box::new(selected(
            "https://source_url/1",
        ))))),
        &ctx,
    );
    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &scale_msg, &ctx);
    assert!(effects.has_changed);
    assert!(effects.is_empty());
    assert_eq!(player.video_scale, Some(VideoScale::Cover));

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Load(ActionLoad::Player(Box::new(selected(
            "https://source_url/2",
        ))))),
        &ctx,
    );
    assert_eq!(player.video_scale, Some(VideoScale::Cover));

    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &Msg::Action(Action::Unload), &ctx);
    assert_eq!(player.video_scale, None);
}

#[test]
fn saved_video_scale_seeds_player_session() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let meta_id = "tt123456";
    let video_id = "tt123456:1:1";
    let selected_stream = stream("https://source_url/1", None);
    let selected = Selected {
        stream: selected_stream.clone(),
        stream_request: Some(request(STREAM_RESOURCE_NAME, video_id)),
        meta_request: Some(request(META_RESOURCE_NAME, meta_id)),
        subtitles_path: None,
    };
    let mut ctx = Ctx::default();
    ctx.streams.items.insert(
        StreamsItemKey {
            meta_id: meta_id.to_owned(),
            video_id: video_id.to_owned(),
        },
        StreamsItem {
            stream: selected_stream,
            r#type: "series".to_owned(),
            meta_id: meta_id.to_owned(),
            video_id: video_id.to_owned(),
            meta_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
            stream_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
            state: Some(StreamItemState {
                video_scale: Some(VideoScale::Cover),
                ..Default::default()
            }),
            mtime: DateTime::<Utc>::default(),
        },
    );
    let mut player = Player {
        selected: Some(selected),
        ..Default::default()
    };

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Internal(Internal::StreamsChanged(false)),
        &ctx,
    );

    assert_eq!(
        player
            .stream_state
            .as_ref()
            .and_then(|state| state.video_scale),
        Some(VideoScale::Cover)
    );
    assert_eq!(player.video_scale, Some(VideoScale::Cover));
}

#[test]
fn saved_video_scale_follows_stream_matching_rules() {
    let saved_stream = stream("https://source_url/1", Some("group"));
    let item = StreamsItem {
        stream: saved_stream.clone(),
        r#type: "series".to_owned(),
        meta_id: "tt123456".to_owned(),
        video_id: "tt123456:1:1".to_owned(),
        meta_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
        stream_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
        state: Some(StreamItemState {
            video_scale: Some(VideoScale::Fill),
            ..Default::default()
        }),
        mtime: DateTime::<Utc>::default(),
    };

    assert_eq!(
        item.adjusted_state(&saved_stream)
            .and_then(|state| state.video_scale),
        Some(VideoScale::Fill)
    );
    assert_eq!(
        item.adjusted_state(&stream("https://source_url/2", Some("group")))
            .and_then(|state| state.video_scale),
        Some(VideoScale::Fill)
    );
    assert_eq!(
        item.adjusted_state(&stream("https://source_url/3", None))
            .and_then(|state| state.video_scale),
        None
    );
}

#[test]
fn action_deserializes() {
    let action = serde_json::from_value::<Action>(serde_json::json!({
        "action": "Player",
        "args": {
            "action": "VideoScaleChanged",
            "args": {
                "videoScale": "fill"
            }
        }
    }))
    .expect("Should deserialize video scale action");

    assert!(matches!(
        action,
        Action::Player(ActionPlayer::VideoScaleChanged {
            video_scale: VideoScale::Fill,
        })
    ));
}
