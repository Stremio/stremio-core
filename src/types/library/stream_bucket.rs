use chrono::{DateTime, Utc};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use crate::types::profile::UID;
use crate::types::resource::Stream;

#[serde_as]
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StreamsBucket {
    pub uid: UID,
    #[serde_as(as = "Vec<(_, _)>")]
    pub items: HashMap<(String, String), StreamBucketItem>,
}

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StreamBucketItem {
    pub stream: Stream,
    pub stream_addon_url: Url,
    pub last_modified: DateTime<Utc>,
}

impl StreamsBucket {
    pub fn new(uid: UID) -> Self {
        StreamsBucket {
            uid,
            items: HashMap::new(),
        }
    }
    pub fn merge_bucket(&mut self, bucket: StreamsBucket) {
        if self.uid == bucket.uid {
            self.items
                .extend(bucket.items.into_iter().map(|(k, v)| (k, v)));
        };
    }
}
