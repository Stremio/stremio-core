#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct Subtitles {
    pub lang: String,
    #[cfg_attr(
        test,
        derivative(Default(value = "Url::parse(\"protocol://host\").unwrap()"))
    )]
    pub url: Url,
}
