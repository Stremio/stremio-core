use crate::types::resource::Stream;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreamsItem {
    pub stream: Stream,
    pub transport_url: Url,
    pub mtime: DateTime<Utc>,
}
