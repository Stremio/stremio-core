use serde_test::{assert_de_tokens, assert_tokens, Configure, Token};
use url::Url;

use crate::{
    types::addon::{Descriptor, DescriptorFlags, Manifest},
    unit_tests::serde::default_tokens_ext::DefaultTokens,
};

#[test]
fn descriptor() {
    assert_tokens(
        &Descriptor {
            manifest: Manifest::default(),
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags::default(),
        }
        .compact(),
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
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags::default(),
        }
        .compact(),
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
