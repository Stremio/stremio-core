use std::collections::HashMap;

#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::Url;

/// See <https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/subtitles.md> for documentation
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct Subtitles {
    pub id: String,
    pub lang: String,
    #[cfg_attr(
        test,
        derivative(Default(value = "Url::parse(\"protocol://host\").unwrap()"))
    )]
    pub url: Url,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fonts: Vec<Url>,
    /// Any other properties the add-on sent, kept rather than dropped, as
    /// `StreamBehaviorHints::other` and `MetaItemBehaviorHints::other` are.
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
