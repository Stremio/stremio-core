use crate::types::resource::PosterShape;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn poster_shape() {
    assert_tokens(
        &vec![
            PosterShape::Square,
            PosterShape::Landscape,
            PosterShape::Poster,
        ],
        &[
            Token::Seq { len: Some(3) },
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("square"),
            Token::Unit,
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("landscape"),
            Token::Unit,
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("poster"),
            Token::Unit,
            Token::SeqEnd,
        ],
    );
    assert_de_tokens(
        &PosterShape::Poster,
        &[
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("invalid"),
            Token::Unit,
        ],
    );
}
