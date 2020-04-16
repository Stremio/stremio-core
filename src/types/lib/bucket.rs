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
    pub fn split_by_recent(&self) -> (LibBucketBorrowed, LibBucketBorrowed) {
        let sorted_items = self
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .collect::<Vec<_>>();
        let recent_count = cmp::min(LIBRARY_RECENT_COUNT, sorted_items.len());
        let (recent_items, other_items) = sorted_items.split_at(recent_count);
        (
            LibBucketBorrowed::new(&self.uid, recent_items),
            LibBucketBorrowed::new(&self.uid, other_items),
        )
    }
}

#[derive(Serialize)]
pub struct LibBucketBorrowed<'a> {
    pub uid: &'a UID,
    pub items: HashMap<&'a str, &'a LibItem>,
}

impl<'a> LibBucketBorrowed<'a> {
    pub fn new(uid: &'a UID, items: &[&'a LibItem]) -> Self {
        LibBucketBorrowed {
            uid,
            items: items.iter().map(|&item| (item.id.as_str(), item)).collect(),
        }
    }
}
