use super::ServerUrlItem;
use crate::{
    constants::{
        SERVER_URL_BUCKET_DEFAULT_ITEM_ID, SERVER_URL_BUCKET_MAX_ITEMS, STREAMING_SERVER_URL,
    },
    types::profile::UID,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServerUrlsBucket {
    /// User ID
    pub uid: UID,
    /// [`HashMap`] Key is the [`ServerUrlItem`]`.id`.
    pub items: HashMap<usize, ServerUrlItem>,
}

impl ServerUrlsBucket {
    /// Create a new [`ServerUrlsBucket`] with the base URL inserted.
    pub fn new(uid: UID) -> Self {
        let mut items = HashMap::new();
        let base_url: &Url = &STREAMING_SERVER_URL;

        let server_base_url_item = ServerUrlItem {
            id: SERVER_URL_BUCKET_DEFAULT_ITEM_ID,
            url: base_url.clone(),
            mtime: Self::current_timestamp() as i64,
            selected: true,
        };

        items.insert(server_base_url_item.id, server_base_url_item);

        ServerUrlsBucket { uid, items }
    }

    fn current_timestamp() -> u64 {
        chrono::Utc::now().timestamp() as u64
    }

    pub fn generate_new_id(&self) -> usize {
        self.items.keys().max().cloned().unwrap_or(0) + 1
    }

    pub fn merge_bucket(&mut self, bucket: ServerUrlsBucket) {
        if self.uid == bucket.uid {
            self.merge_items(bucket.items.into_values().collect());
        }
    }

    pub fn add_url(&mut self, url: Url) {
        let new_id = self.generate_new_id();
        let new_item = ServerUrlItem::new(new_id, url, Self::current_timestamp() as i64);
        self.merge_items(vec![new_item]);
    }

    pub fn merge_items(&mut self, items: Vec<ServerUrlItem>) {
        for new_item in items.into_iter() {
            match self.items.get_mut(&new_item.id) {
                Some(item) => {
                    *item = new_item;
                }
                None => {
                    if self.items.len() < SERVER_URL_BUCKET_MAX_ITEMS {
                        self.items.insert(new_item.id.to_owned(), new_item);
                    } else {
                        let oldest_item_id_option = self
                            .items
                            .values()
                            .filter(|item| item.id != SERVER_URL_BUCKET_DEFAULT_ITEM_ID)
                            .min_by_key(|item| item.mtime)
                            .map(|item| item.id);

                        if let Some(oldest_item_id) = oldest_item_id_option {
                            if new_item.mtime > self.items[&oldest_item_id].mtime {
                                self.items.remove(&oldest_item_id);
                                self.items.insert(new_item.id.to_owned(), new_item);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn edit_item(&mut self, id: &usize, new_url: Url) {
        if let Some(item) = self.items.get_mut(id) {
            item.url = new_url;
            item.mtime = Self::current_timestamp() as i64;
        }
    }

    pub fn delete_item(&mut self, id: &usize) {
        if *id != SERVER_URL_BUCKET_DEFAULT_ITEM_ID {
            self.items.remove(id);
        }
    }
}
