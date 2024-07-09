pub mod deep_links_ext;

mod serialize_catalogs_with_extra;
use serialize_catalogs_with_extra::*;

mod serialize_continue_watching_preview;
use serialize_continue_watching_preview::*;

mod serialize_ctx;
use serialize_ctx::*;

mod serialize_discover;
use serialize_discover::*;

mod serialize_installed_addons;
use serialize_installed_addons::*;

mod serialize_library;
use serialize_library::*;

mod serialize_local_search;
use serialize_local_search::*;

mod serialize_meta_details;
use serialize_meta_details::*;

mod serialize_player;
use serialize_player::*;

mod serialize_remote_addons;
use serialize_remote_addons::*;

mod serialize_streaming_server;
use serialize_streaming_server::*;

mod serialize_data_export;
use serialize_data_export::*;

mod model;
pub use model::*;
