use crate::types::addon::{Descriptor, DescriptorFlags, Manifest, TransportUrl};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_de_tokens, assert_tokens, Token};
use url::Url;

#[test]
fn descriptor() {
    assert_tokens(
        &Descriptor {
            manifest: Manifest::default(),
            transport_url: TransportUrl::parse("https://transport_url.com/manifest.json").unwrap(),
            flags: DescriptorFlags::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "Descriptor",
                    len: 3,
                },
                Token::Str("manifest"),
            ],
            Manifest::default_tokens(),
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
    assert_de_tokens(
        &Descriptor {
            manifest: Manifest::default(),
            transport_url: TransportUrl::parse("https://transport_url.com/manifest.json").unwrap(),
            flags: DescriptorFlags::default(),
        },
        &[
            vec![
                Token::Struct {
                    name: "Descriptor",
                    len: 2,
                },
                Token::Str("manifest"),
            ],
            Manifest::default_tokens(),
            vec![
                Token::Str("transportUrl"),
                Token::Str("https://transport_url/"),
                Token::StructEnd,
            ],
        ]
        .concat(),
    )
}
