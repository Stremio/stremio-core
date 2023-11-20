use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::profile::UID;

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
