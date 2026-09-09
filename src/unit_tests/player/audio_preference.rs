use chrono::{DateTime, Utc};

use crate::{
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::{
        msg::{Action, ActionLoad, ActionPlayer, Msg},
        UpdateWithCtx,
    },
    types::{
        player::AudioPreference,
        resource::{Stream, StreamBehaviorHints, StreamSource},
        streams::{AudioTrack, StreamItemState, StreamsItem},
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

#[test]
fn preference_is_scoped_to_player_session() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let mut player = Player::default();
    let ctx = Ctx::default();
    let preference = AudioPreference {
        language: Some("jpn".to_owned()),
    };
    let preference_msg = Msg::Action(Action::Player(ActionPlayer::AudioPreferenceChanged {
        preference: preference.to_owned(),
    }));

    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &preference_msg, &ctx);
    assert!(!effects.has_changed);
    assert!(effects.is_empty());
    assert_eq!(player.audio_preference, None);

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Load(ActionLoad::Player(Box::new(selected(
            "https://source_url/1",
        ))))),
        &ctx,
    );
    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &preference_msg, &ctx);
    assert!(effects.has_changed);
    assert!(effects.is_empty());
    assert_eq!(player.audio_preference, Some(preference.to_owned()));

    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &preference_msg, &ctx);
    assert!(!effects.has_changed);

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Load(ActionLoad::Player(Box::new(selected(
            "https://source_url/2",
        ))))),
        &ctx,
    );
    assert_eq!(player.audio_preference, Some(preference));

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Player(ActionPlayer::AudioPreferenceChanged {
            preference: AudioPreference { language: None },
        })),
        &ctx,
    );
    assert_eq!(
        player.audio_preference,
        Some(AudioPreference { language: None })
    );

    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &Msg::Action(Action::Unload), &ctx);
    assert_eq!(player.audio_preference, None);
}

#[test]
fn saved_audio_language_follows_stream_matching_rules() {
    for language in [None, Some("jpn".to_owned())] {
        let saved_stream = stream("https://source_url/1", Some("group"));
        let audio_track = AudioTrack {
            id: "EMBEDDED_1".to_owned(),
            language: language.to_owned(),
        };
        let item = StreamsItem {
            stream: saved_stream.clone(),
            r#type: "series".to_owned(),
            meta_id: "tt123456".to_owned(),
            video_id: "tt123456:1:1".to_owned(),
            meta_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
            stream_transport_url: "https://transport_url/manifest.json".parse().unwrap(),
            state: Some(StreamItemState {
                audio_track: Some(audio_track.to_owned()),
                audio_delay: Some(250),
                playback_speed: Some(1.25),
                ..Default::default()
            }),
            mtime: DateTime::<Utc>::default(),
        };

        assert_eq!(item.adjusted_state(&saved_stream), item.state);

        let next_state = item
            .adjusted_state(&stream("https://source_url/2", Some("group")))
            .expect("Should carry adjusted state for the next episode");
        assert_eq!(next_state.audio_track, language.map(|_| audio_track));
        assert_eq!(next_state.audio_delay, None);
        assert_eq!(next_state.playback_speed, Some(1.25));

        for binge_group in [None, Some("other-group")] {
            let next_state = item
                .adjusted_state(&stream("https://source_url/3", binge_group))
                .expect("Should carry playback settings for another source");
            assert_eq!(next_state.audio_track, None);
            assert_eq!(next_state.playback_speed, Some(1.25));
        }
    }
}

#[test]
fn action_deserializes_with_or_without_language() {
    for preference in [
        AudioPreference {
            language: Some("jpn".to_owned()),
        },
        AudioPreference { language: None },
    ] {
        let action = serde_json::from_value::<Action>(serde_json::json!({
            "action": "Player",
            "args": {
                "action": "AudioPreferenceChanged",
                "args": {
                    "preference": preference,
                }
            }
        }))
        .expect("Should deserialize audio preference action");

        assert!(matches!(
            action,
            Action::Player(ActionPlayer::AudioPreferenceChanged {
                preference: parsed_preference,
            }) if parsed_preference == preference
        ));
    }
}
