use serde::{Deserialize, Serialize};

use super::rating;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub status: Option<rating::Status>,
}
