use serde_test::{assert_tokens, Configure, Token};
use url::Url;

use crate::{
    types::addon::{DescriptorFlags, DescriptorPreview, ManifestPreview},
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};

#[test]
fn descriptor_preview() {
    assert_tokens(
        &DescriptorPreview {
            manifest: ManifestPreview::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: Default::default(),
        }
        .compact(),
        &[
            vec![
                Token::Struct {
                    name: "DescriptorPreview",
                    len: 3,
                },
                Token::Str("manifest"),
            ],
            ManifestPreview::default_tokens(),
            vec![
                Token::Str("transportUrl"),
                Token::Str("https://transport_url/"),
                Token::Str("flags"),
            ],
            DescriptorFlags::default_tokens(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
}
