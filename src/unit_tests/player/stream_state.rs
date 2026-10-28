use chrono::{DateTime, Utc};

use crate::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Internal, Msg},
        UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest},
        player::VideoScale,
        resource::{Stream, StreamBehaviorHints, StreamSource},
        streams::{StreamItemState, StreamsItem, StreamsItemKey, SubtitleTrack},
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

fn streams_item(stream: Stream, state: StreamItemState) -> StreamsItem {
    StreamsItem {
        stream,
        r#type: "series".to_owned(),
        meta_id: "tt123456".to_owned(),
        video_id: "tt123456:1:1".to_owned(),
        meta_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
        stream_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
        state: Some(state),
        mtime: DateTime::<Utc>::default(),
    }
}

#[test]
fn saved_stream_state_seeds_player() {
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
        streams_item(
            selected_stream,
            StreamItemState {
                video_scale: Some(VideoScale::Cover),
                ..Default::default()
            },
        ),
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
}

#[test]
fn saved_video_scale_follows_stream_matching_rules() {
    let saved_stream = stream("https://source_url/1", Some("group"));
    let item = streams_item(
        saved_stream.clone(),
        StreamItemState {
            video_scale: Some(VideoScale::Fill),
            ..Default::default()
        },
    );

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
fn saved_subtitles_follow_stream_matching_rules() {
    let saved_stream = stream("https://source_url/1", Some("group"));
    let addon_track = SubtitleTrack {
        id: "https://subtitles_url/1.srt".to_owned(),
        embedded: false,
        language: Some("eng".to_owned()),
    };
    let item = streams_item(
        saved_stream.clone(),
        StreamItemState {
            subtitle_track: Some(addon_track.clone()),
            subtitles_enabled: Some(false),
            ..Default::default()
        },
    );

    let same_source = item.adjusted_state(&saved_stream).unwrap();
    assert_eq!(same_source.subtitle_track, Some(addon_track.clone()));
    assert_eq!(same_source.subtitles_enabled, Some(false));

    let binge_match = item
        .adjusted_state(&stream("https://source_url/2", Some("group")))
        .unwrap();
    assert_eq!(binge_match.subtitle_track, Some(addon_track));
    assert_eq!(binge_match.subtitles_enabled, Some(false));

    let other_stream = item
        .adjusted_state(&stream("https://source_url/3", None))
        .unwrap();
    assert_eq!(other_stream.subtitle_track, None);
    assert_eq!(other_stream.subtitles_enabled, None);

    let item = streams_item(
        saved_stream,
        StreamItemState {
            subtitle_track: Some(SubtitleTrack {
                id: "https://subtitles_url/1.srt".to_owned(),
                embedded: false,
                language: None,
            }),
            ..Default::default()
        },
    );
    assert_eq!(
        item.adjusted_state(&stream("https://source_url/2", Some("group")))
            .and_then(|state| state.subtitle_track),
        None
    );
}
