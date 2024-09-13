use serde::{Deserialize, Serialize};
use serde_with::serde_as;

mod device_info;
pub use device_info::*;

mod network_info;
pub use network_info::*;

mod response;
pub use response::*;

mod request;
pub use request::*;

mod settings;
pub use settings::*;

mod statistics;
pub use statistics::*;

use super::resource::SeriesInfo;
use crate::types::{torrent::InfoHash, DefaultOnBool};

///
/// # Examples
///
/// ```
/// use stremio_core::types::streaming_server::CreatedTorrent;
/// let json = serde_json::json!({
///     "torrent": {
///         "infoHash": "df389295484b3059a4726dc6d8a57f71bb5f4c81",
///     },
///     "peerSearch": { "min": 40, "max": 100, "sources": ["dht:df389295484b3059a4726dc6d8a57f71bb5f4c81", "https://exmaple.com/source"]},
///     "guessFileIdx": false,
/// });
///
/// let created_torrent = serde_json::from_value::<CreatedTorrent>(json).expect("Should deserialize");
/// assert!(created_torrent.guess_file_idx.is_none());
/// ```
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedTorrent {
    pub torrent: Torrent,
    pub peer_search: PeerSearch,
    /// Make the server guess the `fileIdx` based on [`SeriesInfo`].
    ///
    /// `stremio-video` sends `false` when no Guessing should be done.
    /// If `None, the server will perform no guessing
    #[serde_as(deserialize_as = "DefaultOnBool")]
    #[serde(default)]
    pub guess_file_idx: Option<SeriesInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Torrent {
    pub info_hash: InfoHash,
}
