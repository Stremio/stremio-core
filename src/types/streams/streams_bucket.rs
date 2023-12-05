use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::profile::UID;
use crate::types::resource::MetaItem;
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

    pub fn last_stream_item(&self, video_id: &str, meta_item: &MetaItem) -> Option<&StreamsItem> {
        match meta_item.videos.len() {
            0 => self.items.get(&StreamsItemKey {
                meta_id: meta_item.preview.id.to_string(),
                video_id: video_id.to_owned(),
            }),
            _ => meta_item
                .videos
                .iter()
                .position(|video| video.id == video_id)
                .and_then(|max_index| {
                    meta_item.videos[max_index.saturating_sub(30)..=max_index]
                        .iter()
                        .rev()
                        .find_map(|video| {
                            self.items.get(&StreamsItemKey {
                                meta_id: meta_item.preview.id.to_string(),
                                video_id: video.id.to_string(),
                            })
                        })
                }),
        }
    }
}
