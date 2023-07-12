mod env;
pub use env::*;

pub mod test_env;

pub mod catalog_with_filters;
pub mod ctx;
pub mod deep_links;
pub mod meta_details;
pub mod serde;

pub mod data_export;
pub mod link;

pub fn is_send<T: Send>() -> bool {
    true
}
pub fn is_sync<T: Sync>() -> bool {
    true
}
