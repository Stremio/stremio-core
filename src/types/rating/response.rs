use serde::{Deserialize, Serialize};

use crate::types::rating::rating::Rating;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingGetStatusResponse {
    pub status: Option<Rating>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSendResponseRating {
    pub status: Rating,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSendResponse {
    pub message: String,
    pub rating: Option<RatingSendResponseRating>,
    pub action: Option<String>,
}
