use crate::types::profile::{Auth, Profile, Settings};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_de_tokens, assert_tokens, Token};

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
        ],
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "Profile",
                    len: 3,
                },
                Token::Str("auth"),
                Token::Some,
            ],
            Auth::default_tokens(),
            vec![
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("settings"),
            ],
            Settings::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "Profile",
                    len: 3,
                },
                Token::Str("auth"),
                Token::None,
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
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
        },
        &[
            vec![
                Token::Struct {
                    name: "Profile",
                    len: 2,
                },
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("settings"),
            ],
            Settings::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
