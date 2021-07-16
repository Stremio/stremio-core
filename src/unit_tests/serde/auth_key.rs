use crate::types::profile::AuthKey;
use serde_test::{assert_tokens, Token};

#[test]
fn auth_key() {
    assert_tokens(
        &AuthKey("auth_key".to_owned()),
        &[
            Token::NewtypeStruct { name: "AuthKey" },
            Token::Str("auth_key"),
        ],
    );
}
