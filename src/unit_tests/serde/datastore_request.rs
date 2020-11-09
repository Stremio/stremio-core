use crate::types::api::{DatastoreCommand, DatastoreRequest};
use crate::types::profile::AuthKey;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_ser_tokens, Token};

#[test]
fn datastore_request() {
    assert_ser_tokens(
        &DatastoreRequest {
            auth_key: AuthKey::default(),
            collection: "collection".to_owned(),
            command: DatastoreCommand::default(),
        },
        &[
            vec![Token::Map { len: None }, Token::Str("authKey")],
            AuthKey::default_tokens(),
            vec![Token::Str("collection"), Token::Str("collection")],
            vec![Token::MapEnd],
        ]
        .concat(),
    );
}
