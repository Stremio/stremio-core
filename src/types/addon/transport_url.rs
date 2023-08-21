use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// An Addon transport Url `stremio://.../manifest.json` or `https://.../manifest.json`
///
/// When deserializing the url:
/// - Should start with either `stremio://` or `https://`
/// - Should end with `manifest.json`
///
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "Url", into = "Url")]
pub struct TransportUrl(Url);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Only stremio:// and https:// are allowed
    #[error("Url scheme is not supported, only stremio:// and https:// are allowed.")]
    UnsupportedScheme,
    #[error("Manifest.json path is missing from the Url")]
    MissingManifest,
    #[error("Invalid Url")]
    ParsingUrl(#[from] url::ParseError),
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

        if !sanitized_url.path().ends_with("manifest.json") {
            return Err(Error::MissingManifest);
        }

        Ok(Self(sanitized_url))
    }
}

impl FromStr for TransportUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = s.parse::<Url>()?;

        Self::try_from(url)
    }
}

impl From<TransportUrl> for Url {
    fn from(transport_url: TransportUrl) -> Self {
        transport_url.0
    }
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
            assert_eq!(expected_url, transport_url.0)
        }

        // stremio:// protocol w/out manifest.json
        {
            let url = "stremio://addon_url.com"
                .parse::<Url>()
                .expect("Should parse");

            let result = TransportUrl::try_from(url);
            assert_eq!(Err(Error::MissingManifest), result)
        }

        // https:// protocol w/ manifest.json
        {
            let url = "https://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url.clone()).expect("Should convert");
            assert_eq!(url, transport_url.0)
        }

        // https:// protocol w/out manifest.json
        {
            let url = "https://addon_url.com"
                .parse::<Url>()
                .expect("Should parse");

            let result = TransportUrl::try_from(url);
            assert_eq!(Err(Error::MissingManifest), result)
        }

        // http:// protocol w/ manifest.json
        {
            let url = "http://addon_url.com/manifest.json"
                .parse::<Url>()
                .expect("Should parse");

            let transport_url = TransportUrl::try_from(url.clone()).expect("Should convert");
            assert_eq!(url, transport_url.0)
        }

        // https:// protocol w/out manifest.json
        {
            let url = "http://addon_url.com".parse::<Url>().expect("Should parse");

            let result = TransportUrl::try_from(url);
            assert_eq!(Err(Error::MissingManifest), result)
        }
    }
}
