use crate::types::profile::UID;
use crate::types::streams::StreamsItem;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
