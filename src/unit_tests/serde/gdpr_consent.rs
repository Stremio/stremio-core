use crate::types::profile::GDPRConsent;
use serde_test::{assert_tokens, Token};

#[test]
fn gdpr_consent() {
    assert_tokens(
        &GDPRConsent {
            tos: true,
            privacy: true,
            marketing: true,
            from: Some("tests".to_owned()),
        },
        &[
            Token::Struct {
                name: "GDPRConsent",
                len: 4,
            },
            Token::Str("tos"),
            Token::Bool(true),
            Token::Str("privacy"),
            Token::Bool(true),
            Token::Str("marketing"),
            Token::Bool(true),
            Token::Str("from"),
            Token::Some,
            Token::Str("tests"),
            Token::StructEnd,
        ],
    );
}
