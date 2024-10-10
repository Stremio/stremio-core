use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// Server URL Item
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerUrlItem {
    /// Unique ID
    pub id: usize,
    /// URL
    pub url: Url,
    /// Modification time
    pub mtime: DateTime<Utc>,
}

impl ServerUrlItem {
    pub fn new(id: usize, url: Url, mtime: DateTime<Utc>) -> Self {
        ServerUrlItem { id, url, mtime }
    }
}
