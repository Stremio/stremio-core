use http::header;
use url::Url;

use crate::types::streaming_server::split_basic_auth;

#[test]
fn without_credentials_the_url_and_headers_are_untouched() {
    let url = Url::parse("https://host/stremio/").unwrap();
    let (clean, auth) = split_basic_auth(&url);
    assert_eq!(clean, url);
    assert!(auth.is_none());
}

#[test]
fn credentials_are_moved_to_a_basic_authorization_header() {
    let url = Url::parse("https://user:pass@host/stremio/").unwrap();
    let (clean, auth) = split_basic_auth(&url);
    assert_eq!(clean.as_str(), "https://host/stremio/");
    assert_eq!(
        auth.unwrap(),
        "Basic dXNlcjpwYXNz",
        "Authorization header value"
    );
}

#[test]
fn percent_encoded_credentials_are_decoded() {
    let url = Url::parse("https://user%40name:p%3Ass@host/").unwrap();
    let (clean, auth) = split_basic_auth(&url);
    assert_eq!(clean.as_str(), "https://host/");
    assert_eq!(auth.unwrap(), "Basic dXNlckBuYW1lOnA6c3M=");
}

#[test]
fn a_username_without_a_password_still_authenticates() {
    let url = Url::parse("https://onlyuser@host/").unwrap();
    let (clean, auth) = split_basic_auth(&url);
    assert_eq!(clean.as_str(), "https://host/");
    assert_eq!(auth.unwrap(), "Basic b25seXVzZXI6");
}

#[test]
fn apply_basic_auth_adds_the_header_only_when_present() {
    let request = http::Request::get("https://host/settings")
        .body(())
        .unwrap();
    let request = crate::types::streaming_server::apply_basic_auth(request, None);
    assert!(request.headers().get(header::AUTHORIZATION).is_none());

    let auth = header::HeaderValue::from_static("Basic dXNlcjpwYXNz");
    let request = crate::types::streaming_server::apply_basic_auth(
        http::Request::get("https://host/settings")
            .body(())
            .unwrap(),
        Some(&auth),
    );
    assert_eq!(request.headers().get(header::AUTHORIZATION), Some(&auth));
}
