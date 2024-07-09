use crate::types::profile::{Auth, AuthKey, User};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Configure, Token};

#[test]
fn auth() {
    assert_tokens(
        &Auth {
            key: AuthKey::default(),
            user: User::default(),
        }
        .compact(),
        &[
            vec![
                Token::Struct {
                    name: "Auth",
                    len: 2,
                },
                Token::Str("key"),
            ],
            AuthKey::default_tokens(),
            vec![Token::Str("user")],
            User::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
