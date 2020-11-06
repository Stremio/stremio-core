use crate::types::api::{APIError, APIResult};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn api_result() {
    assert_ser_tokens(
        &vec![
            APIResult::Err {
                error: APIError::default(),
            },
            APIResult::Ok { result: () },
        ],
        &[
            vec![
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "APIResult",
                    len: 1,
                },
                Token::Str("error"),
            ],
            APIError::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "APIResult",
                    len: 1,
                },
                Token::Str("result"),
                Token::Unit,
                Token::StructEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
}
