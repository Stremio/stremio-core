use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use crate::types::resource::Stream;

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StreamsItem {
    pub stream: Stream,
    pub transport_url: Url,
    pub mtime: DateTime<Utc>,
}
