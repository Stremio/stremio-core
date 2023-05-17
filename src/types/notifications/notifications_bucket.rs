use std::collections::HashMap;

use crate::runtime::Env;
use crate::types::notifications::NotificationItem;
use crate::types::profile::UID;

use chrono::{DateTime, Utc};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Default, Derivative, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct NotificationsBucket {
    pub uid: UID,
    pub items: HashMap<String, NotificationItem>,
    #[derivative(Default(value = "Utc::now()"))]
    pub created: DateTime<Utc>,
}

impl NotificationsBucket {
    pub fn new<E: Env + 'static>(uid: UID, items: Vec<NotificationItem>) -> Self {
        NotificationsBucket {
            uid,
            items: items
                .into_iter()
                .map(|item| (item.id.to_owned(), item))
                .collect(),
            created: E::now(),
        }
    }
}
