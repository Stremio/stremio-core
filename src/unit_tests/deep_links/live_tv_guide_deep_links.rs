use crate::deep_links::LiveTvGuideDeepLinks;
use crate::types::addon::{ResourcePath, ResourceRequest};
use chrono::NaiveDate;
use std::str::FromStr;
use url::Url;

#[test]
fn live_tv_guide_deep_links() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let ldl = LiveTvGuideDeepLinks::from(&date);
    assert_eq!(
        ldl.live_tv_guide,
        "stremio:///livetv/2026-07-02".to_string()
    );

    let request = ResourceRequest {
        base: Url::from_str("https://addon/manifest.json").unwrap(),
        path: ResourcePath::without_extra("catalog", "tv", "guide"),
    };
    let ldl = LiveTvGuideDeepLinks::from((&request, &date));
    assert_eq!(
        ldl.live_tv_guide,
        "stremio:///livetv/https%3A%2F%2Faddon%2Fmanifest.json/tv/guide/2026-07-02".to_string()
    );
}
