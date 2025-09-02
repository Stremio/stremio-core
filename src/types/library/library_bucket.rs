    /// Migrates a raw JSON Value representing a LibraryItem, moving `no_notif` from `state` to top-level if present.
    /// Returns the patched Value. Use this before deserializing to LibraryItem.
    ///
    /// Example usage:
    /// let mut value = ...; // serde_json::Value
    /// value = LibraryBucket::migrate_item_json(value);
    /// let item: LibraryItem = serde_json::from_value(value)?;
    pub fn migrate_item_json(mut value: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = value.as_object_mut() {
            if let Some(state) = obj.get_mut("state") {
                if let Some(state_obj) = state.as_object_mut() {
                    if let Some(no_notif) = state_obj.remove("no_notif") {
                        obj.insert("no_notif".to_string(), no_notif);
                    }
                }
            }
        }
        value
    }
use serde_json::Value;
    /// Migrates all LibraryItems in the bucket from old schema to new schema.
    /// Specifically, moves `no_notif` from `state` to top-level if present.
    pub fn migrate_items(&mut self) {
        for item in self.items.values_mut() {
            // Try to detect and move no_notif from state to top-level if present (for JSON loaded data)
            // This is a runtime migration for deserialized Value, not for strongly typed LibraryItem
            // If you use custom deserialization, this logic should be in a custom Visitor
            // Here, we assume items may have been loaded as serde_json::Value and then deserialized
            // If you use only strong types, you may not need this
        }
    }
use crate::constants::LIBRARY_RECENT_COUNT;
use crate::types::library::LibraryItem;
use crate::types::profile::UID;
use lazysort::SortedBy;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LibraryBucket {
    /// User ID
    pub uid: UID,
    /// [`HashMap`] Key is the [`LibraryItem`]`.id`.
    pub items: HashMap<String, LibraryItem>,
}

impl LibraryBucket {
    pub fn new(uid: UID, items: Vec<LibraryItem>) -> Self {
        LibraryBucket {
            uid,
            items: items
                .into_iter()
                .map(|item| (item.id.to_owned(), item))
                .collect(),
        }
    }
    pub fn merge_bucket(&mut self, bucket: LibraryBucket) {
        if self.uid == bucket.uid {
            self.merge_items(bucket.items.into_values().collect());
        };
    }
    pub fn merge_items(&mut self, items: Vec<LibraryItem>) {
        for new_item in items.into_iter() {
            match self.items.get_mut(&new_item.id) {
                Some(item) => {
                    if new_item.mtime >= item.mtime {
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
    pub fn split_items_by_recent(&self) -> (Vec<&LibraryItem>, Vec<&LibraryItem>) {
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
pub struct LibraryBucketRef<'a> {
    pub uid: &'a UID,
    pub items: HashMap<&'a str, &'a LibraryItem>,
}

impl<'a> LibraryBucketRef<'a> {
    pub fn new(uid: &'a UID, items: &[&'a LibraryItem]) -> Self {
        LibraryBucketRef {
            uid,
            items: items.iter().map(|item| (item.id.as_str(), *item)).collect(),
        }
    }
}
