use super::ServerUrlItem;
use crate::{
    constants::{SERVER_URL_BUCKET_DEFAULT_ITEM_ID, SERVER_URL_BUCKET_MAX_ITEMS},
    types::profile::UID,
};
use anyhow::{anyhow, Result};
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
    pub fn new(uid: UID, base_url: Url) -> Self {
        let mut items = HashMap::new();

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

    pub fn merge_bucket(&mut self, bucket: ServerUrlsBucket) {
        if self.uid == bucket.uid {
            self.merge_items(bucket.items.into_values().collect());
        }
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

    pub fn edit_item(&mut self, id: &usize, new_url: Url) -> Result<()> {
        if let Some(item) = self.items.get_mut(id) {
            item.url = new_url;
            item.mtime = Self::current_timestamp() as i64;
            Ok(())
        } else {
            Err(anyhow!("Item not found"))
        }
    }

    pub fn delete_item(&mut self, id: &usize) -> Result<()> {
        if *id == SERVER_URL_BUCKET_DEFAULT_ITEM_ID {
            return Err(anyhow!("Cannot remove the base URL item."));
        }
        if self.items.remove(id).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Item not found"))
        }
    }

    pub fn select_item(&mut self, id: &usize) -> Result<()> {
        if let Some(current_selected_item) = self.items.values_mut().find(|item| item.selected) {
            current_selected_item.selected = false;
        }

        if let Some(new_selected_item) = self.items.get_mut(id) {
            new_selected_item.selected = true;
            Ok(())
        } else {
            Err(anyhow!("Item not found"))
        }
    }

    pub fn selected_item(&self) -> Option<&ServerUrlItem> {
        self.items.values().find(|item| item.selected)
    }

    pub fn selected_item_url(&self) -> Option<Url> {
        self.selected_item().map(|item| item.url.clone())
    }
}
