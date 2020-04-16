use super::LibItem;
use crate::constants::LIBRARY_RECENT_COUNT;
use crate::types::profile::UID;
use lazysort::SortedBy;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibBucket {
    pub uid: UID,
    pub items: HashMap<String, LibItem>,
}

impl LibBucket {
    pub fn new(uid: UID, items: Vec<LibItem>) -> Self {
        LibBucket {
            uid,
            items: items
                .into_iter()
                .map(|item| (item.id.to_owned(), item))
                .collect(),
        }
    }
    pub fn merge(&mut self, other: LibBucket) {
        if self.uid != other.uid {
            return;
        }

        for (id, item) in other.items.into_iter() {
            match self.items.entry(id) {
                Vacant(entry) => {
                    entry.insert(item);
                }
                Occupied(mut entry) => {
                    if item.mtime > entry.get().mtime {
                        entry.insert(item);
                    }
                }
            }
        }
    }
    pub fn split_items_by_recent(&self) -> (Vec<&LibItem>, Vec<&LibItem>) {
        let sorted_items = self
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .collect::<Vec<_>>();
        let recent_count = cmp::min(LIBRARY_RECENT_COUNT, sorted_items.len());
        let (recent_items, other_items) = sorted_items.split_at(recent_count);
        (
            recent_items.iter().map(|item| *item).collect(),
            other_items.iter().map(|item| *item).collect(),
        )
    }
}
