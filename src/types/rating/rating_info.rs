use serde::{Deserialize, Serialize};

use crate::types::{rating::Status, resource::MetaItemId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingInfo {
    pub meta_id: MetaItemId,
    pub status: Option<Status>,
}
