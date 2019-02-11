use serde_derive::*;
use chrono::{Utc, DateTime};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    pub fb_id: String,
    pub avatar: String,
    pub last_modified: DateTime<Utc>,
    pub date_registered: DateTime<Utc>,
    // @TODO: others?
    // https://github.com/Stremio/stremio-api/blob/master/types/user.go
}
