use crate::{
    constants::{SERVER_URL_BUCKET_MAX_ITEMS, STREAMING_SERVER_URL},
    runtime::Env,
    types::profile::UID,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServerUrlsBucket {
    /// User ID
    pub uid: UID,
    /// HashMap where key is `Url`, value is `DateTime<Utc>`
    pub items: HashMap<Url, DateTime<Utc>>,
}

impl ServerUrlsBucket {
    /// Create a new `ServerUrlsBucket` with the base URL inserted.
    pub fn new<E: Env + 'static>(uid: UID) -> Self {
        let mut items = HashMap::new();
        let base_url: &Url = &STREAMING_SERVER_URL;
        let mtime = E::now();
        items.insert(base_url.clone(), mtime);

        ServerUrlsBucket { uid, items }
    }

    /// Add a new URL to the bucket.
    pub fn add_url<E: Env + 'static>(&mut self, url: Url) {
        let mtime = E::now();
        self.merge_items(vec![(url, mtime)]);
    }

    /// Merge multiple URL items into the bucket.
    pub fn merge_items(&mut self, items: Vec<(Url, DateTime<Utc>)>) {
        for (url, mtime) in items.into_iter() {
            if let Some(existing_mtime) = self.items.get_mut(&url) {
                *existing_mtime = mtime;
            } else if self.items.len() < SERVER_URL_BUCKET_MAX_ITEMS {
                self.items.insert(url.clone(), mtime);
            } else {
                let default_url: &Url = &STREAMING_SERVER_URL;

                let oldest_item_option = self
                    .items
                    .iter()
                    .filter(|(item_url, _)| *item_url != default_url)
                    .min_by_key(|(_, &item_mtime)| item_mtime)
                    .map(|(item_url, _)| item_url.clone());

                if let Some(oldest_url) = oldest_item_option {
                    if mtime > *self.items.get(&oldest_url).unwrap() {
                        self.items.remove(&oldest_url);
                        self.items.insert(url.clone(), mtime);
                    }
                }
            }
        }
    }

    /// Delete a URL from the bucket.
    pub fn delete_url(&mut self, url: &Url) {
        let default_url: &Url = &STREAMING_SERVER_URL;
        if url != default_url {
            self.items.remove(url);
        }
    }
}
