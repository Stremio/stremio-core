use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subtitles {
    pub id: String,
    // @TODO: ISO 639-2
    pub lang: String,
    pub url: String,
}
