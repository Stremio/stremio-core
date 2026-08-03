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
        player::{SubtitlePreference, SubtitleSource},
        resource::{Stream, StreamBehaviorHints, StreamSource},
    },
    unit_tests::TestEnv,
};

fn selected(source_url: &str) -> Selected {
    Selected {
        stream: Stream {
            source: StreamSource::Url {
                url: source_url.parse().unwrap(),
            },
            name: None,
            description: None,
            thumbnail: None,
            subtitles: vec![],
            behavior_hints: StreamBehaviorHints::default(),
        },
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
    let preference = SubtitlePreference {
        enabled: false,
        source: Some(SubtitleSource::External),
    };
    let preference_msg = Msg::Action(Action::Player(ActionPlayer::SubtitlePreferenceChanged {
        preference,
    }));

    let effects = <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &preference_msg, &ctx);
    assert!(!effects.has_changed);
    assert!(effects.is_empty());
    assert_eq!(player.subtitle_preference, None);

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
    assert_eq!(player.subtitle_preference, Some(preference));

    <Player as UpdateWithCtx<TestEnv>>::update(
        &mut player,
        &Msg::Action(Action::Load(ActionLoad::Player(Box::new(selected(
            "https://source_url/2",
        ))))),
        &ctx,
    );
    assert_eq!(player.subtitle_preference, Some(preference));

    <Player as UpdateWithCtx<TestEnv>>::update(&mut player, &Msg::Action(Action::Unload), &ctx);
    assert_eq!(player.subtitle_preference, None);
}

#[test]
fn action_deserializes() {
    let action = serde_json::from_value::<Action>(serde_json::json!({
        "action": "Player",
        "args": {
            "action": "SubtitlePreferenceChanged",
            "args": {
                "preference": {
                    "enabled": true,
                    "source": "embedded"
                }
            }
        }
    }))
    .expect("Should deserialize subtitle preference action");

    assert!(matches!(
        action,
        Action::Player(ActionPlayer::SubtitlePreferenceChanged {
            preference: SubtitlePreference {
                enabled: true,
                source: Some(SubtitleSource::Embedded),
            }
        })
    ));
}
