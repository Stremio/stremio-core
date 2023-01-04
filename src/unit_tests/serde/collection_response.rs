#[cfg(test)]
mod cft_test {

    use crate::types::api::CollectionResponse;
    use chrono::{TimeZone, Utc};
    use serde_test::{assert_ser_tokens, Token};

    #[test]
    fn collection_response() {
        let last_modified = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

        assert_ser_tokens(
            &CollectionResponse {
                addons: vec![],
                last_modified,
            },
            &[
                Token::Struct {
                    name: "CollectionResponse",
                    len: 2,
                },
                Token::Str("addons"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("lastModified"),
                Token::Str("2020-01-01T00:00:00Z"),
                Token::StructEnd,
            ],
        );
    }
}
