use crate::types::addon::{Descriptor, DescriptorFlags, Manifest};
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_de_tokens, assert_tokens, Token};
use url::Url;

#[test]
fn descriptor() {
    assert_tokens(
        &Descriptor {
            manifest: Manifest::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
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
            Manifest::default_token(),
            vec![
                Token::Str("transportUrl"),
                Token::Str("https://transport_url/"),
                Token::Str("flags"),
            ],
            DescriptorFlags::default_token(),
            vec![Token::StructEnd],
        ]
        .concat(),
    );
    assert_de_tokens(
        &Descriptor {
            manifest: Manifest::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
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
            Manifest::default_token(),
            vec![
                Token::Str("transportUrl"),
                Token::Str("https://transport_url/"),
                Token::StructEnd,
            ],
        ]
        .concat(),
    )
}
