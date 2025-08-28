use crate::{constants::STREAMING_SERVER_URL, runtime::Env, types::profile::UID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServerUrlsBucket {
    /// User ID
    pub uid: UID,
    /// A map of the the server Url as a key and the modified/added datetime of that particular url.
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

    /// Add a URL to the bucket.
    pub fn add_url<E: Env + 'static>(&mut self, url: Url) {
        let mtime = E::now();
        self.items.insert(url, mtime);
    }

    /// Delete a URL from the bucket.
    pub fn delete_url(&mut self, url: &Url) {
        let default_url: &Url = &STREAMING_SERVER_URL;
        if url != default_url {
            self.items.remove(url);
        }
    }
}
