use crate::types::resource::PosterShape;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn ser_de_poster_shape_square() {
    assert_tokens(
        &PosterShape::Square,
        &[
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("square"),
            Token::Unit,
        ],
    );
}

#[test]
fn ser_de_poster_shape_landscaoe() {
    assert_tokens(
        &PosterShape::Landscape,
        &[
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("landscape"),
            Token::Unit,
        ],
    );
}

#[test]
fn ser_de_poster_shape_poster() {
    assert_tokens(
        &PosterShape::Poster,
        &[
            Token::Enum {
                name: "PosterShape",
            },
            Token::Str("poster"),
            Token::Unit,
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
