use serde_json::json;
use serde_test::{assert_de_tokens, assert_tokens, Configure, Token};

use crate::{
    types::{
        addon::{Descriptor, Manifest},
        profile::{Auth, Profile, Settings},
    },
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};

#[test]
fn profile() {
    assert_tokens(
        &vec![
            Profile {
                auth: Some(Auth::default()),
                addons: vec![],
                addons_locked: false,
                settings: Settings::default(),
            },
            Profile {
                auth: None,
                addons: vec![],
                addons_locked: false,
                settings: Settings::default(),
            },
        ]
        .readable(),
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "Profile",
                    len: 4,
                },
                Token::Str("auth"),
                Token::Some,
            ],
            Auth::default_tokens(),
            vec![
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("addonsLocked"),
                Token::Bool(false),
                Token::Str("settings"),
            ],
            Settings::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "Profile",
                    len: 4,
                },
                Token::Str("auth"),
                Token::None,
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("addonsLocked"),
                Token::Bool(false),
                Token::Str("settings"),
            ],
            Settings::default_tokens(),
            vec![Token::StructEnd, Token::SeqEnd],
        ]
        .concat(),
    );
    assert_de_tokens(
        &Profile {
            auth: None,
            addons: vec![],
            addons_locked: false,
            settings: Settings::default(),
        }
        .readable(),
        &[
            vec![
                Token::Struct {
                    name: "Profile",
                    len: 3,
                },
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("addonsLocked"),
                Token::Bool(false),
                Token::Str("settings"),
            ],
            Settings::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}

#[test]
fn saved_profile_skips_invalid_addons_and_keeps_valid_entries_in_order() {
    let first = Descriptor {
        transport_url: "https://first.example/manifest.json".parse().unwrap(),
        manifest: Manifest {
            name: "First addon".to_owned(),
            ..Default::default()
        },
        flags: Default::default(),
    };
    let second = Descriptor {
        transport_url: "https://second.example/manifest.json".parse().unwrap(),
        manifest: Manifest {
            name: "Second addon".to_owned(),
            ..Default::default()
        },
        flags: Default::default(),
    };
    let expected = Profile {
        auth: Some(Auth::default()),
        addons: vec![first.clone(), second.clone()],
        addons_locked: true,
        settings: Settings {
            subtitles_size: 72,
            ..Default::default()
        },
    };
    let mut stored = serde_json::to_value(&expected).unwrap();
    let mut bad = serde_json::to_value(&first).unwrap();
    bad["manifest"]["version"] = json!("invalid-version");
    let mut duplicate = serde_json::to_value(&first).unwrap();
    duplicate["manifest"]["name"] = json!("Duplicate addon");
    stored["addons"] = json!([bad, first, duplicate, second]);

    let stored_json = stored.to_string();
    assert_eq!(
        serde_json::from_str::<Profile>(&stored_json).unwrap(),
        expected
    );
}
