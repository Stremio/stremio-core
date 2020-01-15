use super::LibItem;
use crate::constants::LIBRARY_RECENT_COUNT;
use lazysort::SortedBy;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UID(pub Option<String>);

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
                Occupied(entry) => {
                    if item.mtime > entry.get().mtime {
                        entry.insert(item);
                    }
                }
            }
        }
    }
    pub fn split_by_recent(&self) -> (LibBucketBorrowed, LibBucketBorrowed) {
        let (recent, other) = self
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .collect::<Vec<_>>()
            .split_at(LIBRARY_RECENT_COUNT);
        (
            LibBucketBorrowed::new(&self.uid, recent),
            LibBucketBorrowed::new(&self.uid, other),
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
