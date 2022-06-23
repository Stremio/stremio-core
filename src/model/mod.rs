pub mod deep_links_ext;

mod serialize_catalogs_with_extra;
use serialize_catalogs_with_extra::*;

mod serialize_continue_watching_preview;
use serialize_continue_watching_preview::*;

mod serialize_discover;
use serialize_discover::*;

mod serialize_installed_addons;
use serialize_installed_addons::*;

mod serialize_library;
use serialize_library::*;

mod serialize_meta_details;
use serialize_meta_details::*;

mod serialize_player;
use serialize_player::*;

mod serialize_remote_addons;
use serialize_remote_addons::*;

mod model;
pub use model::*;
