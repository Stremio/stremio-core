use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use crate::types::resource::Stream;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamsItem {
    pub stream: Stream,
    pub r#type: String,
    pub meta_id: String,
    pub video_id: String,
    pub meta_transport_url: Url,
    pub stream_transport_url: Url,
    /// Modification time
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
}
