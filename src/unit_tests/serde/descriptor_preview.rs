use serde_test::{assert_tokens, Token};

use crate::types::addon::{DescriptorPreview, ManifestPreview, TransportUrl};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;

#[test]
fn descriptor_preview() {
    assert_tokens(
        &DescriptorPreview {
            manifest: ManifestPreview::default(),
            transport_url: TransportUrl::parse("https://transport_url.com/manifest.json").unwrap(),
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
