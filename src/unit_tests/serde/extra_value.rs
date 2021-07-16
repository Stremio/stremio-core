use crate::types::addon::ExtraValue;
use serde_test::{assert_tokens, Token};

#[test]
fn extra_value() {
    assert_tokens(
        &ExtraValue {
            name: "name".to_owned(),
            value: "value".to_owned(),
        },
        &[
            Token::Tuple { len: 2 },
            Token::Str("name"),
            Token::Str("value"),
            Token::TupleEnd,
        ],
    );
}
