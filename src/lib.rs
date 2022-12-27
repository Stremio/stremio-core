#![allow(clippy::module_inception)]
// TODO: Fix async tests that trigger this lock warning
#![cfg_attr(test, allow(clippy::await_holding_lock))]

pub mod addon_transport;
pub mod deep_links;
pub mod models;
pub mod runtime;
pub mod types;

pub mod constants;

#[cfg(test)]
pub(crate) mod unit_tests;
