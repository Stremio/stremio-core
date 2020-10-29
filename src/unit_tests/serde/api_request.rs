use crate::types::api::{APIRequest, AuthRequest};
use crate::types::profile::AuthKey;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn api_request() {
    assert_ser_tokens(
        &vec![
            APIRequest::Auth(AuthRequest::default()),
            APIRequest::Logout {
                auth_key: AuthKey::default(),
            },
            APIRequest::AddonCollectionGet {
                auth_key: AuthKey::default(),
                update: true,
            },
            APIRequest::AddonCollectionSet {
                auth_key: AuthKey::default(),
                addons: vec![],
            },
        ],
        &[
            vec![Token::Seq { len: Some(4) }],
            AuthRequest::default_tokens(),
            vec![
                Token::Struct {
                    name: "APIRequest",
                    len: 2,
                },
                Token::Str("type"),
                Token::Str("Logout"),
                Token::Str("authKey"),
            ],
            AuthKey::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "APIRequest",
                    len: 3,
                },
                Token::Str("type"),
                Token::Str("AddonCollectionGet"),
                Token::Str("authKey"),
            ],
            AuthKey::default_tokens(),
            vec![
                Token::Str("update"),
                Token::Bool(true),
                Token::StructEnd,
                Token::Struct {
                    name: "APIRequest",
                    len: 3,
                },
                Token::Str("type"),
                Token::Str("AddonCollectionSet"),
                Token::Str("authKey"),
            ],
            AuthKey::default_tokens(),
            vec![
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
}
