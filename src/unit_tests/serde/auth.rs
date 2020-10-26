use crate::types::profile::{Auth, AuthKey, User};
use crate::unit_tests::serde::default_token_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};

#[test]
fn auth() {
    assert_tokens(
        &Auth {
            key: AuthKey::default(),
            user: User::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "Auth",
                    len: 2,
                },
                Token::Str("key"),
            ],
            AuthKey::default_token(),
            vec![Token::Str("user")],
            User::default_token(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
