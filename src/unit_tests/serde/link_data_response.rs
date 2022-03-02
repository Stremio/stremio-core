use crate::types::api::LinkDataResponse;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn link_data_response() {
    assert_ser_tokens(
        &LinkDataResponse::AuthKey {
            auth_key: "AUTH_KEY".to_owned(),
            link: "LINK".to_owned(),
            qrcode: "QRCODE".to_owned(),
        },
        &[
            Token::Struct {
                name: "LinkDataResponse",
                len: 3,
            },
            Token::Str("authKey"),
            Token::Str("AUTH_KEY"),
            Token::Str("link"),
            Token::Str("LINK"),
            Token::Str("qrcode"),
            Token::Str("QRCODE"),
            Token::StructEnd,
        ],
    );
}
