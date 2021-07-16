use crate::types::True;
use serde_test::{assert_tokens, Token};

#[test]
fn r#true() {
    assert_tokens(&True, &[Token::Bool(true)]);
}
