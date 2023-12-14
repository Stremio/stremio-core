use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::types::profile::UID;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DismissedEventsBucket {
    pub uid: UID,
    /// the key is the event (modal or notification) that has been dismissed
    /// while the value is the time at which it was dismissed.
    pub items: HashMap<String, DateTime<Local>>,
}

impl DismissedEventsBucket {
    pub fn new(uid: UID) -> Self {
        Self {
            uid,
            items: HashMap::new(),
        }
    }
}
