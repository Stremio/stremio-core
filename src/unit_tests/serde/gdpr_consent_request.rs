use crate::types::api::GDPRConsentRequest;
use crate::types::profile::GDPRConsent;
use crate::unit_tests::serde::default_tokens_ext::DefaultFlattenTokens;
use chrono::prelude::TimeZone;
use chrono::Utc;
use serde_test::{assert_tokens, Token};

#[test]
fn gdpr_consent_request() {
    assert_tokens(
        &GDPRConsentRequest {
            gdpr_consent: GDPRConsent::default(),
            time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            from: "from".to_owned(),
        },
        &[
            vec![Token::Map { len: None }],
            GDPRConsent::default_flatten_tokens(),
            vec![
                Token::Str("time"),
                Token::Str("2020-01-01T00:00:00Z"),
                Token::Str("from"),
                Token::Str("from"),
                Token::MapEnd,
            ],
        ]
        .concat(),
    );
}
