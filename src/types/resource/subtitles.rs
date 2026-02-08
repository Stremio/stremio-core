#[cfg(test)]
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::Url;

/// Subtitle track for a video stream.
///
/// Contains the subtitle file URL and language information.
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
    #[serde(with = "crate::types::serde_ext::url_serde")]
    pub url: Url,
}

impl Subtitles {
    /// Resolves a relative URL to an absolute URL using the given base URL.
    ///
    /// If the subtitle URL is relative (doesn't have a host), it will be
    /// joined with the base URL to create an absolute URL.
    ///
    /// # Arguments
    /// * `base_url` - The addon's base URL to use for resolving relative paths
    pub fn resolve_relative_urls(&mut self, base_url: &Url) {
        // If the URL has "relative" scheme, it means it was a relative path
        // that we forced into a URL during deserialization.
        if self.url.scheme() == "relative" {
            // relative:///path -> path is /path
            // relative:path -> path is path
            if let Ok(absolute) = base_url.join(self.url.path()) {
                self.url = absolute;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_relative_url() {
        let base_url = Url::parse("https://addon.example.com/manifest.json").unwrap();
        
        // Create a subtitle with a relative-like path using relative: scheme
        // This simulates what our custom deserializer produces
        let mut subtitle = Subtitles {
            id: "sub1".to_string(),
            lang: "en".to_string(),
            url: Url::parse("relative:///subtitles/english.srt").unwrap(),
        };

        subtitle.resolve_relative_urls(&base_url);

        assert_eq!(
            subtitle.url.as_str(),
            "https://addon.example.com/subtitles/english.srt"
        );
    }

    #[test]
    fn test_absolute_url_unchanged() {
        let base_url = Url::parse("https://addon.example.com/manifest.json").unwrap();
        let mut subtitle = Subtitles {
            id: "sub1".to_string(),
            lang: "en".to_string(),
            url: Url::parse("https://cdn.example.com/subtitles/english.srt").unwrap(),
        };

        subtitle.resolve_relative_urls(&base_url);

        // Absolute URL should remain unchanged
        assert_eq!(
            subtitle.url.as_str(),
            "https://cdn.example.com/subtitles/english.srt"
        );
    }
}
