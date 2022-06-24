use crate::deep_links::query_params_encode::query_params_encode;

#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
pub struct ErrorLink(String);

impl From<anyhow::Error> for ErrorLink {
    fn from(error: anyhow::Error) -> Self {
        Self(format!(
            "stremio:///error?{}",
            query_params_encode(&[("message", error.to_string())]),
        ))
    }
}

impl Into<String> for ErrorLink {
    fn into(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorLink;

    #[test]
    fn load_action() {
        let link = ErrorLink::from(anyhow::Error::msg("message"));
        assert_eq!(
            link,
            ErrorLink("stremio:///error?message=message".to_owned())
        );
    }
}
