use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::profile::UID;

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHistoryBucket {
    pub uid: UID,
    pub items: HashMap<String, DateTime<Utc>>,
}

impl SearchHistoryBucket {
    pub fn new(uid: UID) -> Self {
        Self {
            uid,
            items: HashMap::new(),
        }
    }
}
