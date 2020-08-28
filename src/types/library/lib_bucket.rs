use crate::constants::LIBRARY_RECENT_COUNT;
use crate::types::library::LibItem;
use crate::types::profile::UID;
use lazysort::SortedBy;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
    pub fn merge(&mut self, items: Vec<LibItem>) {
        for new_item in items.into_iter() {
            match self.items.get_mut(&new_item.id) {
                Some(item) => {
                    if new_item.mtime > item.mtime {
                        *item = new_item;
                    }
                }
                None => {
                    self.items.insert(new_item.id.to_owned(), new_item);
                }
            }
        }
    }
    pub fn are_ids_in_recent(&self, ids: &[String]) -> bool {
        let recent_item_ids = self
            .items
            .iter()
            .sorted_by(|(_, a), (_, b)| b.mtime.cmp(&a.mtime))
            .map(|(id, _)| id)
            .take(LIBRARY_RECENT_COUNT)
            .collect::<HashSet<_>>();
        ids.iter().all(move |id| recent_item_ids.contains(id))
    }
    pub fn split_items_by_recent(&self) -> (Vec<&LibItem>, Vec<&LibItem>) {
        let sorted_items = self
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .collect::<Vec<_>>();
        let recent_count = cmp::min(LIBRARY_RECENT_COUNT, sorted_items.len());
        let (recent_items, other_items) = sorted_items.split_at(recent_count);
        (recent_items.to_vec(), other_items.to_vec())
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
            items: items.iter().map(|item| (item.id.as_str(), *item)).collect(),
        }
    }
}
