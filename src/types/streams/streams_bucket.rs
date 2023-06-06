use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::profile::UID;
use crate::types::streams::StreamsItem;

#[serde_as]
#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StreamsBucket {
    pub uid: UID,
    #[serde_as(as = "Vec<(_, _)>")]
    pub items: HashMap<(String, String), StreamsItem>,
}

impl StreamsBucket {
    pub fn new(uid: UID) -> Self {
        StreamsBucket {
            uid,
            items: HashMap::new(),
        }
    }
}
