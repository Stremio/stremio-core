use super::LibItem;
use crate::state_types::models::Auth;
use lazysort::SortedBy;
use serde_derive::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

// According to a mid-2019 study, only 2.7% of users
// have a library larger than that
pub const LIB_RECENT_COUNT: usize = 200;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct UID(Option<String>);
impl From<Option<&Auth>> for UID {
    fn from(a: Option<&Auth>) -> Self {
        UID(a.map(|a| a.user.id.to_owned()))
    }
}

// The LibBucket is a representation of a "bucket" of LibItems for a particular user
// It provides a "try_merge" method, which will only merge the second bucket, if it's for the same
// UID, and it will only merge the newer items
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct LibBucket {
    pub uid: UID,
    pub items: HashMap<String, LibItem>,
}
impl LibBucket {
    pub fn new(uid: UID, items: Vec<LibItem>) -> Self {
        LibBucket {
            uid,
            items: items.into_iter().map(|i| (i.id.to_owned(), i)).collect(),
        }
    }
    pub fn try_merge(&mut self, other: LibBucket) -> &Self {
        if self.uid != other.uid {
            return self;
        }

        for (k, item) in other.items.into_iter() {
            match self.items.entry(k) {
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

        self
    }
    pub fn split_by_recent(&self) -> (LibBucketBorrowed, LibBucketBorrowed) {
        let sorted = self
            .items
            .values()
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .collect::<Vec<&_>>();
        let (recent, other) = sorted.split_at(LIB_RECENT_COUNT);

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
            items: items.iter().map(|i| (i.id.as_str(), *i)).collect(),
        }
    }
}
