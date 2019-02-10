use serde_derive::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    pub fb_id: String,
    pub avatar: String,
    // @TODO: others?
    // https://github.com/Stremio/stremio-api/blob/master/types/user.go
}
