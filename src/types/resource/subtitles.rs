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
    /// Any other properties the add-on sent along with the subtitle.
    ///
    /// The protocol only specifies `id`, `url` and `lang`, but add-ons
    /// commonly attach extra properties that a client can make good use of,
    /// e.g. the OpenSubtitles v3 add-on sends `subtitleFileName`,
    /// `movieReleaseName`, `releaseGroup` and `fpsMilli` on every subtitle.
    /// Instead of blessing one add-on's property names, we keep whatever was
    /// sent, so that it reaches the UI rather than being silently dropped.
    ///
    /// Same approach as [`StreamBehaviorHints::other`] and
    /// [`MetaItemBehaviorHints::other`].
    ///
    /// [`StreamBehaviorHints::other`]: crate::types::resource::StreamBehaviorHints::other
    /// [`MetaItemBehaviorHints::other`]: crate::types::resource::MetaItemBehaviorHints::other
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
