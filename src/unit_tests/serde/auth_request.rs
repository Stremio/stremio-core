use crate::types::api::AuthRequest;
use crate::types::profile::GDPRConsent;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};

#[test]
fn auth_request() {
    assert_tokens(
        &vec![
            AuthRequest::Login {
                email: "email".to_owned(),
                password: "password".to_owned(),
                facebook: false,
            },
            AuthRequest::LoginWithToken {
                token: "token".to_owned(),
            },
            AuthRequest::Facebook {
                token: "token".to_owned(),
            },
            AuthRequest::Apple {
                token: "token".to_owned(),
                sub: "sub".to_owned(),
                email: "email".to_owned(),
                name: "name".to_owned(),
            },
            AuthRequest::Register {
                email: "email".to_owned(),
                password: "password".to_owned(),
                gdpr_consent: GDPRConsent::default(),
            },
        ],
        &[
            vec![
                Token::Seq { len: Some(5) },
                Token::Struct {
                    name: "AuthRequest",
                    len: 4,
                },
                Token::Str("type"),
                Token::Str("Login"),
                Token::Str("email"),
                Token::Str("email"),
                Token::Str("password"),
                Token::Str("password"),
                Token::Str("facebook"),
                Token::Bool(false),
                Token::StructEnd,
                Token::Struct {
                    name: "AuthRequest",
                    len: 2,
                },
                Token::Str("type"),
                Token::Str("LoginWithToken"),
                Token::Str("token"),
                Token::Str("token"),
                Token::StructEnd,
                Token::Struct {
                    name: "AuthRequest",
                    len: 2,
                },
                Token::Str("type"),
                Token::Str("Facebook"),
                Token::Str("token"),
                Token::Str("token"),
                Token::StructEnd,
                Token::Struct {
                    name: "AuthRequest",
                    len: 5,
                },
                Token::Str("type"),
                Token::Str("Apple"),
                Token::Str("token"),
                Token::Str("token"),
                Token::Str("sub"),
                Token::Str("sub"),
                Token::Str("email"),
                Token::Str("email"),
                Token::Str("name"),
                Token::Str("name"),
                Token::StructEnd,
                Token::Struct {
                    name: "AuthRequest",
                    len: 4,
                },
                Token::Str("type"),
                Token::Str("Register"),
                Token::Str("email"),
                Token::Str("email"),
                Token::Str("password"),
                Token::Str("password"),
                Token::Str("gdpr_consent"),
            ],
            GDPRConsent::default_tokens(),
            vec![Token::StructEnd, Token::SeqEnd],
        ]
        .concat(),
    );
}
