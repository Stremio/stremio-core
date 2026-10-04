use base64::Engine;
use http::{header, HeaderValue};
use percent_encoding::percent_decode_str;
use url::Url;

/// Splits `user:password@host` credentials out of a streaming server URL.
///
/// Returns a copy of the URL without the userinfo plus, when credentials are
/// present, the `Authorization: Basic ...` header value to send instead.
/// Credentials cannot stay embedded in the URL: browsers reject fetch URLs
/// that contain them, and HTTP clients do not turn URL userinfo into a header.
pub fn split_basic_auth(url: &Url) -> (Url, Option<HeaderValue>) {
    if url.username().is_empty() && url.password().is_none() {
        return (url.clone(), None);
    }
    let username = percent_decode_str(url.username()).decode_utf8_lossy();
    let password = url
        .password()
        .map(|password| {
            percent_decode_str(password)
                .decode_utf8_lossy()
                .into_owned()
        })
        .unwrap_or_default();
    let credentials = format!("{username}:{password}");
    let mut clean = url.clone();
    let _ = clean.set_username("");
    let _ = clean.set_password(None);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
    let header = HeaderValue::from_str(&format!("Basic {encoded}")).ok();
    (clean, header)
}

/// Adds the `Authorization` header extracted by [`split_basic_auth`] to a
/// request, leaving it untouched when there are no credentials.
pub fn apply_basic_auth<T>(
    mut request: http::Request<T>,
    auth: Option<&HeaderValue>,
) -> http::Request<T> {
    if let Some(auth) = auth {
        request
            .headers_mut()
            .insert(header::AUTHORIZATION, auth.clone());
    }
    request
}
