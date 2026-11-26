use std::borrow::Cow;

#[cfg(test)]
use derivative::Derivative;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use url::Url;

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
}

impl Subtitles {
    /// Returns a subtitle filename when the URL path contains one.
    pub fn filename(&self) -> Option<Cow<'_, str>> {
        let segment = self.url.path_segments()?.next_back()?;
        let filename = percent_decode_str(segment).decode_utf8().ok()?;
        let (stem, extension) = filename.rsplit_once('.')?;
        let is_subtitle = ["srt", "ass", "ssa", "vtt", "sub", "smi", "ttml"]
            .iter()
            .any(|suffix| extension.eq_ignore_ascii_case(suffix));

        if stem.is_empty()
            || !is_subtitle
            || filename
                .chars()
                .any(|character| character.is_control() || matches!(character, '/' | '\\'))
        {
            return None;
        }

        Some(filename)
    }
}
