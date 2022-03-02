use crate::types::api::LinkCodeResponse;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn link_code_response() {
    assert_ser_tokens(
        &LinkCodeResponse {
            code: "CODE".to_owned(),
        },
        &[
            Token::Struct {
                name: "LinkCodeResponse",
                len: 1,
            },
            Token::Str("code"),
            Token::Str("CODE"),
            Token::StructEnd,
        ],
    );
}
