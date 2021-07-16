use crate::types::api::APIError;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn api_error() {
    assert_ser_tokens(
        &APIError {
            message: "message".to_owned(),
            code: 1,
        },
        &[
            Token::Struct {
                name: "APIError",
                len: 2,
            },
            Token::Str("message"),
            Token::Str("message"),
            Token::Str("code"),
            Token::U64(1),
            Token::StructEnd,
        ],
    );
}
