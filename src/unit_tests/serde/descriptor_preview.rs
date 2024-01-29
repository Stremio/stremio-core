use serde_test::{assert_tokens, Configure, Token};
use url::Url;

use crate::{
    types::addon::{DescriptorPreview, ManifestPreview},
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};

#[test]
fn descriptor_preview() {
    assert_tokens(
        &DescriptorPreview {
            manifest: ManifestPreview::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
        }
        .compact(),
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
