#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};

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
