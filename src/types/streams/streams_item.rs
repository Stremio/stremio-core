use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::{addon::ResourceRequest, resource::Stream};

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamsItem {
    pub stream: Stream,
    pub stream_request: ResourceRequest,
    pub meta_request: ResourceRequest,
    /// Modification time
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
}
