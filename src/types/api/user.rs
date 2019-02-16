use chrono::{DateTime, Utc};
use serde_derive::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub fb_id: Option<String>,
    pub avatar: Option<String>,
    pub last_modified: DateTime<Utc>,
    pub date_registered: DateTime<Utc>,
    // @TODO: others?
    // https://github.com/Stremio/stremio-api/blob/master/types/user.go
}
