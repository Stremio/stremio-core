use crate::types::resource::Link;
use serde_test::{assert_tokens, Token};
use url::Url;

#[test]
fn link() {
    assert_tokens(
        &Link {
            name: "name".to_owned(),
            category: "category".to_owned(),
            url: Url::parse("https://url").unwrap(),
        },
        &[
            Token::Struct {
                name: "Link",
                len: 3,
            },
            Token::Str("name"),
            Token::Str("name"),
            Token::Str("category"),
            Token::Str("category"),
            Token::Str("url"),
            Token::Str("https://url/"),
            Token::StructEnd,
        ],
    );
}
