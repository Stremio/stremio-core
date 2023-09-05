use crate::types::api::SuccessResponse;
use crate::types::True;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn success_response() {
    assert_ser_tokens(
        &SuccessResponse { success: True },
        &[
            vec![
                Token::Struct {
                    name: "SuccessResponse",
                    len: 1,
                },
                Token::Str("success"),
            ],
            True::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
