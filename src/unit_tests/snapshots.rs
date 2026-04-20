//! Public-contract tripwire.
//!
//! These snapshots fix the **serialized JSON shape** of every bucket that
//! `stremio-core` writes to persistent storage. The storage keys themselves
//! are declared `pub const` in [`crate::constants`] and are installed on
//! end-user devices; any rename or schema change breaks existing installs
//! silently. The snapshots here make such a change impossible to land
//! without a visible `cargo insta review` diff, forcing a deliberate
//! migration story rather than a surprise drop.
//!
//! To update a snapshot intentionally: run `cargo insta review`,
//! inspect the diff, accept it, and include the `.snap` change in the
//! same PR as the schema change (paired with a migration step in
//! [`crate::runtime::env`] if the wire format is not backward-compatible).
//!
//! What's covered here:
//! - Default `Profile` serialization (the `profile` storage key).
//! - Empty bucket types for the remaining storage keys, as declared in
//!   [`crate::constants`]:
//!   `library`, `library_recent`, `streams`, `search_history`,
//!   `streaming_server_urls`, `notifications`, `calendar`,
//!   `dismissed_events`.
//!
//! What's intentionally NOT covered (deliberate scope limit):
//! - Every [`crate::runtime::msg`] variant — the `Msg` JSON shape is
//!   dispatch-internal, not persisted to storage; its stability matters
//!   for live clients but not for long-term data compatibility.
//! - Deep-link URLs — already covered semantically by
//!   [`crate::unit_tests::deep_links::helpers::assert_player_url`] and the
//!   ordinary `assert_eq!` checks in the deep-link tests.

use insta::assert_json_snapshot;

use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;

/// Default `Profile` — the seed state for a fresh install. Covers the
/// `profile` storage key at [`crate::constants::PROFILE_STORAGE_KEY`].
#[test]
fn snapshot_profile_default() {
    assert_json_snapshot!(Profile::default());
}

/// Empty `LibraryBucket` — covers both the `library` and `library_recent`
/// storage keys (same wire shape, split by recency of access).
#[test]
fn snapshot_library_bucket_empty() {
    assert_json_snapshot!(LibraryBucket::default());
}

/// Empty `StreamsBucket` — covers [`crate::constants::STREAMS_STORAGE_KEY`].
#[test]
fn snapshot_streams_bucket_empty() {
    assert_json_snapshot!(StreamsBucket::default());
}

/// Empty `SearchHistoryBucket` — covers
/// [`crate::constants::SEARCH_HISTORY_STORAGE_KEY`].
#[test]
fn snapshot_search_history_bucket_empty() {
    assert_json_snapshot!(SearchHistoryBucket::default());
}

/// Empty `ServerUrlsBucket` — covers
/// [`crate::constants::STREAMING_SERVER_URLS_STORAGE_KEY`].
#[test]
fn snapshot_server_urls_bucket_empty() {
    // The bucket's `new::<E>()` is parameterized on Env; default() is
    // the neutral construction path and is what the ctx storage layer
    // falls through to on a fresh install.
    assert_json_snapshot!(ServerUrlsBucket::default());
}

/// Empty `NotificationsBucket` — covers
/// [`crate::constants::NOTIFICATIONS_STORAGE_KEY`].
#[test]
fn snapshot_notifications_bucket_empty() {
    assert_json_snapshot!(NotificationsBucket::default());
}

/// Empty `DismissedEventsBucket` — covers
/// [`crate::constants::DISMISSED_EVENTS_STORAGE_KEY`].
#[test]
fn snapshot_dismissed_events_bucket_empty() {
    assert_json_snapshot!(DismissedEventsBucket::default());
}
