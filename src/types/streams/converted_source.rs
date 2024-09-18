use url::Url;

use crate::types::{resource::StreamSource, torrent::InfoHash};

/// Trait which defines the StreamSource state data structures in Core.
pub trait StreamSourceTrait: sealed::Sealed {}
/// only we should be able to define which data structures are StreamSource states!
mod sealed {
    pub trait Sealed {}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConvertedStreamSource {
    Url(Url),
    Torrent {
        url: Url,
        info_hash: InfoHash,
        file_idx: Option<u16>,
        announce: Vec<String>,
    },
}
impl StreamSourceTrait for ConvertedStreamSource {}
impl sealed::Sealed for ConvertedStreamSource {}

impl sealed::Sealed for StreamSource {}
impl StreamSourceTrait for StreamSource {}
