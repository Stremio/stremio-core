use crate::{
    types::{
        addon::{Descriptor, Manifest},
        profile::{Auth, AuthKey, Profile, Settings},
    },
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};
use serde_json::{from_value, to_value, Value};
use serde_test::{assert_de_tokens, assert_ser_tokens, Configure, Token};

#[test]
fn profile_keeps_auth_and_addons_without_saved_settings() {
    let profile = Profile {
        auth: Some(Auth {
            key: AuthKey("saved-session".to_owned()),
            ..Default::default()
        }),
        addons: vec![Descriptor {
            transport_url: "https://addon.example/manifest.json".parse().unwrap(),
            manifest: Manifest::default(),
            flags: Default::default(),
        }],
        addons_locked: true,
        settings: Settings::default(),
    };
    let mut no_settings = to_value(&profile).unwrap();
    assert_eq!(from_value::<Profile>(no_settings.clone()).unwrap(), profile);
    no_settings.as_object_mut().unwrap().remove("settings");
    let mut null_settings = no_settings.clone();
    null_settings["settings"] = Value::Null;

    for payload in [no_settings, null_settings] {
        let restored: Profile = from_value(payload).unwrap();
        assert_eq!(restored, profile);
    }
}

#[test]
fn profile() {
    assert_ser_tokens(
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
                Token::Some,
            ],
            Settings::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
