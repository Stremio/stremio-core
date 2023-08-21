use std::{ops, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Only stremio:// and https:// are allowed
    #[error("Url scheme is not supported, only stremio:// and https:// are allowed.")]
    UnsupportedScheme,
    #[error("Invalid Url")]
    ParsingUrl(#[from] url::ParseError),
}

/// An Addon transport Url `stremio://example_addon.com` or `https://example_addon.com`
///
/// When deserializing the url:
/// - Should start with either `stremio://` or `https://`
///
/// Optionally it can end with `manifest.json` for SDK Addons.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(try_from = "Url", into = "Url")]
pub struct TransportUrl {
    url: Url,
    pub has_manifest: bool,
}

/// The manifest.json filename that we define the TransportUrl with.
const MANIFEST: &str = "manifest.json";

impl TransportUrl {
    ///
    /// # Examples
    ///
    /// ## Parsing
    ///
    /// ```
    /// let with_manifest = TransportUrl::parse("https://v3-cinemeta.strem.io/manifest.json").expect("Cinemeta url parse failed");
    /// assert!(with_manifest.has_manifest);
    ///
    /// let no_manifest = TransportUrl::parse("https://v3-cinemeta.strem.io").expect("Cinemeta url parse failed");
    /// assert!(!no_manifest.has_manifest);
    ///
    ///```
    ///
    pub fn new(addon_url: Url) -> Self {
        Self {
            has_manifest: has_manifest(&addon_url),
            url: addon_url,
        }
    }

    pub fn parse(input: &str) -> Result<Self, Error> {
        input.parse()
    }
}

impl From<TransportUrl> for Url {
    fn from(transport_url: TransportUrl) -> Url {
        transport_url.url
    }
}

impl From<&TransportUrl> for Url {
    fn from(transport_url: &TransportUrl) -> Url {
        transport_url.url.to_owned()
    }
}

impl TryFrom<Url> for TransportUrl {
    type Error = Error;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        let sanitized_url = match url.scheme() {
            "https" | "http" => url,
            // replace stremio:// with https://
            "stremio" => url
                .as_str()
                .replacen("stremio", "https", 1)
                .parse()
                .expect("Should never fail"),
            _ => return Err(Error::UnsupportedScheme),
        };

        let with_manifest = sanitized_url.path().ends_with("manifest.json");

        Ok(Self {
            url: sanitized_url,
            has_manifest: with_manifest,
        })
    }
}

impl ops::Deref for TransportUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl FromStr for TransportUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = s.parse::<Url>()?;

        Self::try_from(url)
    }
}
fn has_manifest(url: &Url) -> bool {
    url.path().ends_with(MANIFEST)
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::{Error, TransportUrl};

    #[test]
    fn test_deserialization_of_transport_url() {
        // stremio:// protocol w/ manifest.json
        {
            let url = "stremio://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");
            let expected_url = "https://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url).expect("Should convert");
            assert_eq!(expected_url, transport_url.url);
            assert!(transport_url.has_manifest);
        }

        // stremio:// protocol w/out manifest.json
        {
            let url = "stremio://addon_url.com"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url).expect("Should parse");
            assert!(!transport_url.has_manifest);
        }

        // https:// protocol w/ manifest.json
        {
            let url = "https://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url.clone()).expect("Should convert");
            assert_eq!(url, transport_url.url);
            assert!(transport_url.has_manifest);
        }

        // https:// protocol w/out manifest.json
        {
            let url = "https://addon_url.com"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url).expect("Should parse");
            assert!(!transport_url.has_manifest);
            assert!(transport_url.has_manifest);
        }

        // http:// protocol w/ manifest.json
        {
            let url = "http://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url.clone()).expect("Should convert");
            assert_eq!(url, transport_url.url);
            assert!(transport_url.has_manifest);
        }

        // https:// protocol w/out manifest.json
        {
            let url = "http://addon_url.com".parse::<Url>().expect("Should parse");

            let transport_url = TransportUrl::try_from(url).expect("Should parse");
            assert!(!transport_url.has_manifest)
        }
    }
}
