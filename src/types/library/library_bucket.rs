use crate::constants::LIBRARY_RECENT_COUNT;
use crate::types::library::LibraryItem;
use crate::types::profile::UID;
use serde::{Deserialize, Serialize, Serializer};
use std::cmp;
use std::collections::HashMap;

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
        if ids.is_empty() {
            return true;
        }
        if self.items.len() <= LIBRARY_RECENT_COUNT {
            return ids.iter().all(|id| self.items.contains_key(id));
        }
        if let [id] = ids {
            return self.items.get(id).is_some_and(|item| {
                self.items
                    .values()
                    .filter(|other| other.mtime >= item.mtime)
                    .count()
                    <= LIBRARY_RECENT_COUNT
            });
        }
        let mut mtimes = self
            .items
            .values()
            .map(|item| item.mtime)
            .collect::<Vec<_>>();
        let (_, threshold, _) =
            mtimes.select_nth_unstable_by(LIBRARY_RECENT_COUNT - 1, |a, b| b.cmp(a));
        let threshold = *threshold;
        let ties_fit =
            mtimes.iter().filter(|mtime| **mtime >= threshold).count() <= LIBRARY_RECENT_COUNT;
        ids.iter().all(|id| {
            self.items
                .get(id)
                .is_some_and(|item| item.mtime > threshold || (item.mtime == threshold && ties_fit))
        })
    }
    pub fn split_items_by_recent(&self) -> (Vec<&LibraryItem>, Vec<&LibraryItem>) {
        let mut recent_items = self.items_partitioned_by_recent();
        let other_items =
            recent_items.split_off(cmp::min(LIBRARY_RECENT_COUNT, recent_items.len()));
        (recent_items, other_items)
    }
    pub(crate) fn recent_items(&self) -> Vec<&LibraryItem> {
        let mut items = self.items_partitioned_by_recent();
        items.truncate(LIBRARY_RECENT_COUNT);
        items
    }
    fn items_partitioned_by_recent(&self) -> Vec<&LibraryItem> {
        let mut items = self.items.values().collect::<Vec<_>>();
        if items.len() > LIBRARY_RECENT_COUNT {
            items.select_nth_unstable_by(LIBRARY_RECENT_COUNT, |a, b| b.mtime.cmp(&a.mtime));
        }
        items
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

#[derive(Serialize)]
pub(crate) struct LibraryBucketSliceRef<'a> {
    pub uid: &'a UID,
    #[serde(serialize_with = "serialize_items_by_id")]
    pub items: &'a [&'a LibraryItem],
}

fn serialize_items_by_id<S: Serializer>(
    items: &&[&LibraryItem],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_map(items.iter().map(|item| (item.id.as_str(), *item)))
}
