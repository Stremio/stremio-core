use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};
use url::Url;

#[test]
fn resource_request() {
    assert_tokens(
        &ResourceRequest {
            base: Url::parse("https://base").unwrap(),
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
