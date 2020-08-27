use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Poster,
    Square,
    Landscape,
    #[serde(other)]
    Unspecified,
}

impl Default for PosterShape {
    fn default() -> Self {
        PosterShape::Unspecified
    }
}

impl PosterShape {
    pub fn is_unspecified(&self) -> bool {
        *self == PosterShape::Unspecified
    }
}
