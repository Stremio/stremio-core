use crate::types::addon::ResourcePath;
use serde_test::{assert_tokens, Token};

#[test]
fn resource_path() {
    assert_tokens(
        &ResourcePath {
            resource: "resource".to_owned(),
            r#type: "type".to_owned(),
            id: "id".into(),
            extra: vec![],
        },
        &[
            Token::Struct {
                name: "ResourcePath",
                len: 4,
            },
            Token::Str("resource"),
            Token::Str("resource"),
            Token::Str("type"),
            Token::Str("type"),
            Token::Str("id"),
            Token::Str("id"),
            Token::Str("extra"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}
