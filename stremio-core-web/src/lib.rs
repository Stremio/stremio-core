#[allow(clippy::module_inception)]
pub mod model;

#[cfg(feature = "wasm")]
pub mod env;
#[cfg(feature = "wasm")]
pub mod event;
#[cfg(feature = "wasm")]
pub mod stremio_core_web;
