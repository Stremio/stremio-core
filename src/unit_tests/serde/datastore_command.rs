use crate::types::api::DatastoreCommand;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn datastore_command() {
    assert_ser_tokens(
        &vec![
            DatastoreCommand::Meta,
            DatastoreCommand::Get {
                ids: vec!["id".to_owned()],
                all: true,
            },
            DatastoreCommand::Put { changes: vec![] },
        ],
        &[
            Token::Seq { len: Some(3) },
            Token::Unit,
            Token::Struct {
                name: "DatastoreCommand",
                len: 2,
            },
            Token::Str("ids"),
            Token::Seq { len: Some(1) },
            Token::Str("id"),
            Token::SeqEnd,
            Token::Str("all"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Struct {
                name: "DatastoreCommand",
                len: 1,
            },
            Token::Str("changes"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
        ],
    );
}
