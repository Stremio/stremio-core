use crate::deep_links::AddonsDeepLinks;
use crate::models::installed_addons_with_filters::InstalledAddonsRequest;
use crate::types::addon::{ResourcePath, ResourceRequest};
use std::str::FromStr;
use url::Url;

#[test]
fn addons_deep_links_installed_addons_request_no_type() {
    let request = InstalledAddonsRequest { r#type: None };
    let adl = AddonsDeepLinks::from(&request);
    assert_eq!(adl.addons, "stremio:///addons".to_string());
}

#[test]
fn addons_deep_links_installed_addons_request_type() {
    let request = InstalledAddonsRequest {
        r#type: Some("movie".to_string()),
    };
    let adl = AddonsDeepLinks::from(&request);
    assert_eq!(adl.addons, "stremio:///addons/movie".to_string());
}

#[test]
fn addons_deep_links_request() {
    let request = ResourceRequest {
        base: Url::from_str("http://v3-cinemeta.strem.io").unwrap(),
        path: ResourcePath::without_extra("addons", "movie", "com.linvo.cinemeta"),
    };
    let adl = AddonsDeepLinks::from(&request);
    assert_eq!(
        adl.addons,
        "stremio:///addons/movie/http%3A%2F%2Fv3-cinemeta.strem.io%2F/com.linvo.cinemeta"
            .to_string()
    );
}
