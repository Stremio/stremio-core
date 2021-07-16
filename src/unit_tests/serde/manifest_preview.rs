use crate::types::addon::ManifestPreview;
use semver::Version;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn manifest_preview() {
    assert_tokens(
        &vec![
            ManifestPreview {
                id: "id".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: Some("description".to_owned()),
                logo: Some("logo".to_owned()),
                background: Some("background".to_owned()),
                types: vec!["type".to_owned()],
            },
            ManifestPreview {
                id: "id".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: None,
                logo: None,
                background: None,
                types: vec![],
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Struct {
                name: "ManifestPreview",
                len: 7,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("version"),
            Token::Str("0.0.1"),
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("description"),
            Token::Some,
            Token::Str("description"),
            Token::Str("logo"),
            Token::Some,
            Token::Str("logo"),
            Token::Str("background"),
            Token::Some,
            Token::Str("background"),
            Token::Str("types"),
            Token::Seq { len: Some(1) },
            Token::Str("type"),
            Token::SeqEnd,
            Token::StructEnd,
            Token::Struct {
                name: "ManifestPreview",
                len: 7,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("version"),
            Token::Str("0.0.1"),
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("description"),
            Token::None,
            Token::Str("logo"),
            Token::None,
            Token::Str("background"),
            Token::None,
            Token::Str("types"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &ManifestPreview {
            id: "id".to_owned(),
            version: Version::new(0, 0, 1),
            name: "name".to_owned(),
            description: None,
            logo: None,
            background: None,
            types: vec![],
        },
        &[
            Token::Struct {
                name: "ManifestPreview",
                len: 4,
            },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("version"),
            Token::Str("0.0.1"),
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("types"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}
