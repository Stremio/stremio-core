#![allow(clippy::all)]
#![allow(deprecated)]
//! # Stremio core
#![allow(clippy::module_inception)]
// Do not allow broken intra doc links
#![deny(rustdoc::broken_intra_doc_links)]
// TODO: Fix async tests that trigger this lock warning
#![cfg_attr(test, allow(clippy::await_holding_lock))]
#![cfg_attr(test, allow(clippy::needless_update))]

// Re-export of the derive macro for Model which removes the need for
// depending on `stremio-derive` everywhere.
#[cfg(feature = "derive")]
pub use stremio_derive::Model;

pub mod addon_transport;
#[cfg(feature = "analytics")]
pub mod analytics;
pub mod deep_links;
pub mod models;
pub mod runtime;
pub mod types;

pub mod constants;

#[cfg(test)]
pub(crate) mod unit_tests;
