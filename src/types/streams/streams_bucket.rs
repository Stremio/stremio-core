use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::profile::UID;
use crate::types::streams::StreamsItem;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamsItemKey {
    pub meta_id: String,
    pub video_id: String,
}

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamsBucket {
    pub uid: UID,
    #[serde_as(as = "Vec<(_, _)>")]
    pub items: HashMap<StreamsItemKey, StreamsItem>,
}

impl StreamsBucket {
    pub fn new(uid: UID) -> Self {
        StreamsBucket {
            uid,
            items: HashMap::new(),
        }
    }
}
