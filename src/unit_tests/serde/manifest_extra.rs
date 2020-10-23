use crate::types::addon::ManifestExtra;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn manifest_extra() {
    assert_tokens(
        &vec![
            ManifestExtra::Full { props: vec![] },
            ManifestExtra::Short {
                required: vec!["required".to_owned()],
                supported: vec!["supported".to_owned()],
            },
            ManifestExtra::Short {
                required: vec![],
                supported: vec![],
            },
        ],
        &[
            Token::Seq { len: Some(3) },
            Token::Struct {
                name: "ManifestExtra",
                len: 1,
            },
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "ManifestExtra",
                len: 2,
            },
            Token::Str("extraRequired"),
            Token::Seq { len: Some(1) },
            Token::Str("required"),
            Token::SeqEnd,
            Token::Str("extraSupported"),
            Token::Seq { len: Some(1) },
            Token::Str("supported"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "ManifestExtra",
                len: 2,
            },
            Token::Str("extraRequired"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("extraSupported"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &ManifestExtra::Short {
            required: vec![],
            supported: vec![],
        },
        &[
            Token::Struct {
                name: "ManifestExtra",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}
