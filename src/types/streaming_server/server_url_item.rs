use serde::{Deserialize, Serialize};
use url::Url;

/// Server URL Item
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerUrlItem {
    /// Unique ID
    pub id: String,
    /// URL
    pub url: Url,
    /// Timestamp
    pub mtime: i64,
    /// Selected
    pub selected: bool,
}

impl ServerUrlItem {
    pub fn new(id: String, url: Url, mtime: i64) -> Self {
        ServerUrlItem {
            id,
            url,
            mtime,
            selected: false,
        }
    }
}
