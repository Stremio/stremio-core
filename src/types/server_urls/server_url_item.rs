use serde::{Deserialize, Serialize};
use url::Url;

/// Server URL Item
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerUrlItem {
    /// Unique ID
    pub id: usize,
    /// URL
    pub url: Url,
    /// Timestamp
    pub mtime: i64,
}

impl ServerUrlItem {
    pub fn new(id: usize, url: Url, mtime: i64) -> Self {
        ServerUrlItem {
            id,
            url,
            mtime,
        }
    }
}
