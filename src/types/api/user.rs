use crate::types::api::GDPRConsent;
use chrono::{DateTime, Utc};
use serde_derive::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub fb_id: Option<String>,
    pub avatar: Option<String>,
    pub last_modified: DateTime<Utc>,
    pub date_registered: DateTime<Utc>,
    #[serde(rename = "gdpr_consent")]
    pub gdpr_consent: GDPRConsent,
    // @TODO: others?
    // https://github.com/Stremio/stremio-api/blob/master/types/user.go
}
