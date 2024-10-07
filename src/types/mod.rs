pub mod addon;
pub mod api;
pub mod events;
pub mod library;
pub mod notifications;
pub mod player;
pub mod profile;
pub mod resource;
pub mod search_history;
pub mod server_urls;
pub mod streaming_server;
pub mod streams;
pub mod torrent;

// Re-export of stremio_watched_bitfield crate
pub mod watched_bitfield {
    #[doc(inline)]
    pub use stremio_watched_bitfield::*;
}

mod query_params_encode;
pub use query_params_encode::*;

mod serde_as_ext;
pub use serde_as_ext::*;

mod r#true;
pub use r#true::*;

mod serde_ext;
pub use serde_ext::*;
