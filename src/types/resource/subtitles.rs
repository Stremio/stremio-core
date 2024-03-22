#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

use super::UrlExtended;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct Subtitles {
    pub lang: String,
    #[cfg_attr(
        test,
        derivative(Default(
            value = "UrlExtended::Url(url::Url::parse(\"protocol://host\").unwrap())"
        ))
    )]
    pub url: UrlExtended,
}

impl Subtitles {
    /// This method will replace the relative path with absolute one using the provided addon transport URL,
    /// only if the url is [`UrlExtended::RelativePath`].
    ///
    /// Otherwise, it leaves the [`Subtitles`] unchanged.
    pub fn with_addon_url(&mut self, addon_transport_url: &Url) -> Result<(), ParseError> {
        self.url.with_addon_url(addon_transport_url)?;

        Ok(())
    }
}
