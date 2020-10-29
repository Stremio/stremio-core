use crate::types::addon::{DescriptorPreview, ManifestPreview};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};
use url::Url;

#[test]
fn descriptor_preview() {
    assert_tokens(
        &DescriptorPreview {
            manifest: ManifestPreview::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
        },
        &[
            vec![
                Token::Struct {
                    name: "DescriptorPreview",
                    len: 2,
                },
                Token::Str("manifest"),
            ],
            ManifestPreview::default_tokens(),
            vec![
                Token::Str("transportUrl"),
                Token::Str("https://transport_url/"),
                Token::StructEnd,
            ],
        ]
        .concat(),
    );
}
