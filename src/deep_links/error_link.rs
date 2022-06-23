use crate::constants::URI_COMPONENT_ENCODE_SET;
use lazy_static::lazy_static;
use percent_encoding::utf8_percent_encode;
use url::Url;

lazy_static! {
    static ref BASE: Url = Url::parse("stremio:///error").expect("ErrorLink BASE parse failed");
}

pub struct ErrorLink(String);

impl From<anyhow::Error> for ErrorLink {
    fn from(error: anyhow::Error) -> Self {
        let query = utf8_percent_encode(
            &format!("?message={}", error.to_string()),
            URI_COMPONENT_ENCODE_SET,
        )
        .to_string();
        Self(BASE.join(&query).unwrap().as_str().to_owned())
    }
}

impl Into<String> for ErrorLink {
    fn into(self) -> String {
        self.0
    }
}
