use crate::types::addon::ManifestBehaviorHints;
use serde_test::{assert_de_tokens, assert_tokens, Token};

#[test]
fn manifest_behavior_hints() {
    assert_tokens(
        &ManifestBehaviorHints {
            adult: true,
            p2p: true,
            configurable: true,
            configuration_required: true,
            epg_provider: true,
        },
        &[
            Token::Struct {
                name: "ManifestBehaviorHints",
                len: 5,
            },
            Token::Str("adult"),
            Token::Bool(true),
            Token::Str("p2p"),
            Token::Bool(true),
            Token::Str("configurable"),
            Token::Bool(true),
            Token::Str("configurationRequired"),
            Token::Bool(true),
            Token::Str("epgProvider"),
            Token::Bool(true),
            Token::StructEnd,
        ],
    );
    assert_de_tokens(
        &ManifestBehaviorHints {
            adult: false,
            p2p: false,
            configurable: false,
            configuration_required: false,
            epg_provider: false,
        },
        &[
            Token::Struct {
                name: "ManifestBehaviorHints",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}
