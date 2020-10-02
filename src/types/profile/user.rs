use crate::types::empty_string_as_none;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct GDPRConsent {
    pub tos: bool,
    pub privacy: bool,
    pub marketing: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub fb_id: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub avatar: Option<String>,
    pub last_modified: DateTime<Utc>,
    pub date_registered: DateTime<Utc>,
    #[serde(rename = "gdpr_consent")]
    pub gdpr_consent: GDPRConsent,
}
