use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::profile::UserId;

use super::like::{self, Status};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub status: Option<like::Status>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SendResult {
    /// Response when 200 status - OK
    Ok(SendOkResponse),
    /// Response when 201 status - Created
    Created(SendCreatedResponse),
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOkResponse {
    pub message: String,
    pub user_id: UserId,
    pub imdb_id: String,
    pub media_type: String,
    /// when status is removed, this will be `"removed"`
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCreatedResponse {
    pub message: String,
    pub rating: CreatedRating,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedRating {
    pub user_id: UserId,
    pub imdb_id: String,
    pub media_type: String,
    /// status is always present for a newly created Rating
    pub status: Status,
    #[serde(default)]
    /// A 2-letter ISO country code (will be converted to uppercase)
    pub country_code: Option<String>,
    pub status_updated_at: DateTime<Utc>,
}
