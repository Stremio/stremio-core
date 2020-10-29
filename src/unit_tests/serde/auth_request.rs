use crate::types::api::{AuthRequest, GDPRConsentRequest};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};

#[test]
fn auth_request() {
    assert_tokens(
        &vec![
            AuthRequest::Login {
                email: "email".to_owned(),
                password: "password".to_owned(),
            },
            AuthRequest::Register {
                email: "email".to_owned(),
                password: "password".to_owned(),
                gdpr_consent: GDPRConsentRequest::default(),
            },
        ],
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "AuthRequest",
                    len: 3,
                },
                Token::Str("type"),
                Token::Str("Login"),
                Token::Str("email"),
                Token::Str("email"),
                Token::Str("password"),
                Token::Str("password"),
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
            GDPRConsentRequest::default_tokens(),
            vec![Token::StructEnd, Token::SeqEnd],
        ]
        .concat(),
    );
}
