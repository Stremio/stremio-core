use crate::types::addon::{ResourcePath, ResourceRequest, TransportUrl};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};

#[test]
fn resource_request() {
    assert_tokens(
        &ResourceRequest {
            base: TransportUrl::parse("https://transport_url.com/manifest.json").unwrap(),
            path: ResourcePath::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "ResourceRequest",
                    len: 2,
                },
                Token::Str("base"),
                Token::Str("https://base/"),
                Token::Str("path"),
            ],
            ResourcePath::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
