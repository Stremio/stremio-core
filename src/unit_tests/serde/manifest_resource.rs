use crate::types::addon::ManifestResource;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn manifest_resource() {
    assert_tokens(
        &vec![
            ManifestResource::Short("resource".to_owned()),
            ManifestResource::Full {
                name: "name".to_owned(),
                types: Some(vec!["type".to_owned()]),
                id_prefixes: Some(vec!["id_prefix".to_owned()]),
            },
            ManifestResource::Full {
                name: "name".to_owned(),
                types: None,
                id_prefixes: None,
            },
        ],
        &[
            Token::Seq { len: Some(3) },
            Token::Str("resource"),
            Token::Struct {
                name: "ManifestResource",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("types"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Str("type"),
            Token::SeqEnd,
            Token::Str("idPrefixes"),
            Token::Some,
            Token::Seq { len: Some(1) },
            Token::Str("id_prefix"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "ManifestResource",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("types"),
            Token::None,
            Token::Str("idPrefixes"),
            Token::None,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &ManifestResource::Full {
            name: "name".to_owned(),
            types: None,
            id_prefixes: None,
        },
        &[
            Token::Struct {
                name: "ManifestResource",
                len: 1,
            },
            Token::Str("name"),
            Token::Str("name"),
            Token::StructEnd,
        ],
    );
}
