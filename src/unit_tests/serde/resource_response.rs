use crate::types::addon::ResourceResponse;
use crate::types::resource::MetaItem;
use crate::unit_tests::serde::default_tokens_ext::DefaultTokens;
use serde_test::{assert_tokens, Token};

#[test]
fn resource_response() {
    assert_tokens(
        &vec![
            ResourceResponse::Metas { metas: vec![] },
            ResourceResponse::MetasDetailed {
                metas_detailed: vec![],
            },
            ResourceResponse::Meta {
                meta: MetaItem::default(),
            },
            ResourceResponse::Streams { streams: vec![] },
            ResourceResponse::Subtitles { subtitles: vec![] },
            ResourceResponse::Addons { addons: vec![] },
        ],
        &[
            vec![
                Token::Seq { len: Some(6) },
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("metas"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("metasDetailed"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("meta"),
            ],
            MetaItem::default_tokens(),
            vec![
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("streams"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("subtitles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Struct {
                    name: "ResourceResponse",
                    len: 1,
                },
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::SeqEnd,
            ],
        ]
        .concat(),
    );
}
