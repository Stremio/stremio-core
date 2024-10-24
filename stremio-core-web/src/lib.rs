#[allow(clippy::module_inception)]
pub mod model {
    #[cfg(feature = "wasm")]
    pub use {
        deep_links_ext::DeepLinksExt, model::*, serialize_calendar::serialize_calendar,
        serialize_continue_watching_preview::serialize_continue_watching_preview,
        serialize_ctx::serialize_ctx, serialize_data_export::serialize_data_export,
        serialize_discover::serialize_discover,
        serialize_installed_addons::serialize_installed_addons,
        serialize_library::serialize_library, serialize_local_search::serialize_local_search,
        serialize_meta_details::serialize_meta_details, serialize_player::serialize_player,
        serialize_remote_addons::serialize_remote_addons,
        serialize_streaming_server::serialize_streaming_server,
    };

    /// Trait which allows you to serialize one struct
    /// on specific platform.
    /// E.g.
    /// - protobuf for kotlin
    /// - JSON for wasm
    /// - etc.
    pub trait SerializeModel<Out> {
        type Error;

        fn serialize_model(&self) -> Result<Out, Self::Error>;
    }

    pub mod deep_links_ext {
        pub trait DeepLinksExt {
            fn into_web_deep_links(self) -> Self;
        }

        mod addons_deep_links;
        mod calendar_deep_links;
        mod discover_deep_links;
        mod library_deep_links;
        mod library_item_deep_links;
        mod local_search_deep_links;
        mod meta_item_deep_links;
        mod search_history_deep_links;
        mod stream_deep_links;
        mod video_deep_links;
    }

    #[cfg(feature = "wasm")]
    mod model;

    pub mod serialize_calendar;
    pub mod serialize_catalogs_with_extra;
    pub mod serialize_continue_watching_preview;
    pub mod serialize_ctx;
    pub mod serialize_data_export;
    pub mod serialize_discover;
    pub mod serialize_installed_addons;
    pub mod serialize_library;
    pub mod serialize_local_search;
    pub mod serialize_meta_details;
    pub mod serialize_player;
    pub mod serialize_remote_addons;
    pub mod serialize_streaming_server;
}

#[cfg(all(feature = "wasm", feature = "env"))]
pub mod env;
#[cfg(feature = "wasm")]
pub mod event;
#[cfg(all(feature = "wasm", feature = "env"))]
mod stremio_core_web;
#[cfg(all(feature = "wasm", feature = "env"))]
// re-export all wasm-specific
pub use stremio_core_web::*;
