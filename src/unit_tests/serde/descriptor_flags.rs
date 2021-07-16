use crate::types::addon::DescriptorFlags;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn descriptor_flags() {
    assert_tokens(
        &DescriptorFlags {
            official: true,
            protected: true,
        },
        &[
            Token::Struct {
                name: "DescriptorFlags",
                len: 2,
            },
            Token::Str("official"),
            Token::Bool(true),
            Token::Str("protected"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &DescriptorFlags {
            official: false,
            protected: false,
        },
        &[
            Token::Struct {
                name: "DescriptorFlags",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}
