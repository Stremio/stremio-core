use crate::types::profile::GDPRConsent;
use serde_test::{assert_tokens, Token};

#[test]
fn gdpr_consent() {
    assert_tokens(
        &GDPRConsent {
            tos: true,
            privacy: true,
            marketing: true,
        },
        &[
            Token::Struct {
                name: "GDPRConsent",
                len: 3,
            },
            Token::Str("tos"),
            Token::Bool(true),
            Token::Str("privacy"),
            Token::Bool(true),
            Token::Str("marketing"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
}
