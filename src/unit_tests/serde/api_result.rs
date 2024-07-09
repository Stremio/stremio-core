use crate::types::api::{APIError, APIResult};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn api_result() {
    assert_ser_tokens(
        &vec![APIResult::Err(APIError::default()), APIResult::Ok(())],
        &[
            vec![Token::Seq { len: Some(2) }],
            vec![Token::NewtypeVariant {
                name: "APIResult",
                variant: "error",
            }],
            APIError::default_tokens(),
            vec![
                Token::NewtypeVariant {
                    name: "APIResult",
                    variant: "result",
                },
                Token::Unit,
            ],
            vec![Token::SeqEnd],
        ]
        .concat(),
    );
}
