use crate::types::api::{LinkAuthKey, LinkDataResponse};
use serde_test::{assert_ser_tokens, Token};

#[test]
fn link_data_response() {
    assert_ser_tokens(
        &LinkDataResponse::AuthKey(LinkAuthKey {
            auth_key: "AUTH_KEY".to_owned(),
        }),
        &[
            Token::Struct {
                name: "LinkAuthKey",
                len: 1,
            },
            Token::Str("authKey"),
            Token::Str("AUTH_KEY"),
            Token::StructEnd,
        ],
    );
}
