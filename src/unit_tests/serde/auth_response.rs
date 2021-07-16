use crate::types::api::AuthResponse;
use crate::types::profile::{AuthKey, User};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn auth_response() {
    assert_ser_tokens(
        &AuthResponse {
            key: AuthKey::default(),
            user: User::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "AuthResponse",
                    len: 2,
                },
                Token::Str("authKey"),
            ],
            AuthKey::default_tokens(),
            vec![Token::Str("user")],
            User::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
