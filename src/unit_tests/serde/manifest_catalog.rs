use crate::types::addon::{ManifestCatalog, ManifestExtra};
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn manifest_catalog() {
    assert_tokens(
        &vec![
            ManifestCatalog {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: Some("name".to_owned()),
                extra: ManifestExtra::default(),
            },
            ManifestCatalog {
                id: "id".into(),
                r#type: "type".to_owned(),
                name: None,
                extra: ManifestExtra::default(),
            },
        ],
        &[
            Token::Seq { len: Some(2) },
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("name"),
            Token::Some,
            Token::Str("name"),
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::MapEnd,
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("name"),
            Token::None,
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::MapEnd,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &ManifestCatalog {
            id: "id".into(),
            r#type: "type".to_owned(),
            name: None,
            extra: ManifestExtra::default(),
        },
        &[
            Token::Map { len: None },
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::MapEnd,
        ],
    );
}
