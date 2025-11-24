use std::collections::HashMap;

use once_cell::sync::Lazy;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use url::Url;

use crate::types::addon::{Descriptor, ExtraProp, OptionsLimit};

pub const SCHEMA_VERSION_STORAGE_KEY: &str = "schema_version";
pub const PROFILE_STORAGE_KEY: &str = "profile";
pub const LIBRARY_STORAGE_KEY: &str = "library";
pub const LIBRARY_RECENT_STORAGE_KEY: &str = "library_recent";
pub const STREAMS_STORAGE_KEY: &str = "streams";
pub const SEARCH_HISTORY_STORAGE_KEY: &str = "search_history";
pub const STREAMING_SERVER_URLS_STORAGE_KEY: &str = "streaming_server_urls";
pub const NOTIFICATIONS_STORAGE_KEY: &str = "notifications";
pub const CALENDAR_STORAGE_KEY: &str = "calendar";
pub const DISMISSED_EVENTS_STORAGE_KEY: &str = "dismissed_events";
pub const LIBRARY_COLLECTION_NAME: &str = "libraryItem";
pub const SEARCH_EXTRA_NAME: &str = "search";
/// `https://{ADDON_UR}/meta/...` resource
pub const META_RESOURCE_NAME: &str = "meta";
pub const STREAM_RESOURCE_NAME: &str = "stream";
/// `https://{ADDON_URL}/catalog/...` resource
pub const CATALOG_RESOURCE_NAME: &str = "catalog";
pub const SUBTITLES_RESOURCE_NAME: &str = "subtitles";
pub const ADDON_MANIFEST_PATH: &str = "/manifest.json";
pub const ADDON_LEGACY_PATH: &str = "/stremio/v1";
pub const CATALOG_PAGE_SIZE: usize = 100;
pub const CATALOG_PREVIEW_SIZE: usize = 100;
pub const LIBRARY_RECENT_COUNT: usize = 200;
pub const NOTIFICATION_ITEMS_COUNT: usize = 100;
/// Maximum calendar items to fetch from `calendarIds` resource
pub const CALENDAR_ITEMS_COUNT: usize = 100;

/// Account age in days to be considered a new user
pub const NEW_USER_DAYS: chrono::Duration = chrono::Duration::days(30);

/// A `LibraryItem` is considered watched once we've watched more than the `duration * threshold`:
///
/// `LibraryItem.state.time_watched` > `LibraryItem.state.duration` * [`WATCHED_THRESHOLD_COEF`]
pub const WATCHED_THRESHOLD_COEF: f64 = 0.7;
pub const CREDITS_THRESHOLD_COEF: f64 = 0.9;
/// The latest migration scheme version
pub const SCHEMA_VERSION: u32 = 20;
pub const IMDB_LINK_CATEGORY: &str = "imdb";
pub const GENRES_LINK_CATEGORY: &str = "Genres";
pub const CINEMETA_TOP_CATALOG_ID: &str = "top";
/// Only found in Cinemeta catalogs, i.e. [`CINEMETA_CATALOGS_URL`]
pub const CINEMETA_FEED_CATALOG_ID: &str = "feed.json";
pub const IMDB_TITLE_PATH: &str = "title";
pub const YOUTUBE_ADDON_ID_PREFIX: &str = "yt_id:";
pub const URI_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

pub const USER_LIKES_SUPPORTED_ID_PREFIXES: &[&str] = &["tt", "tmdb", "kitsu"];

pub const USER_LIKES_SUPPORTED_TYPES: &[&str] = &["movie", "series"];

/// In milliseconds
pub const PLAYER_IGNORE_SEEK_AFTER: u64 = 600_000;

pub static BASE64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD;

pub static CINEMETA_CATALOGS_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://cinemeta-catalogs.strem.io").expect("CINEMETA_URL parse failed")
});

/// Manifest URL for Cinemeta V3
pub static CINEMETA_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://v3-cinemeta.strem.io/manifest.json").expect("CINEMETA_URL parse failed")
});
pub static USER_LIKES_API_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://likes.stremio.com").expect("API_URL parse failed"));
pub static API_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.strem.io").expect("API_URL parse failed"));
pub static LINK_API_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://link.stremio.com").expect("LINK_API_URL parse failed"));
pub static STREAMING_SERVER_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("http://127.0.0.1:11470").expect("STREAMING_SERVER_URL parse failed"));
pub static IMDB_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://imdb.com").expect("IMDB_URL parse failed"));
pub static OFFICIAL_ADDONS: Lazy<Vec<Descriptor>> = Lazy::new(|| {
    let addons_str = stremio_official_addons::get_addons_string();
    serde_json::from_slice(&addons_str).expect("OFFICIAL_ADDONS parse failed")
});
pub static SKIP_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "skip".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit::default(),
});
pub static VIDEO_HASH_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "videoHash".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit::default(),
});
pub static VIDEO_SIZE_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "videoSize".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit::default(),
});
pub static VIDEO_FILENAME_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "filename".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit::default(),
});
pub static LAST_VIDEOS_IDS_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "lastVideosIds".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit(1),
});
pub static CALENDAR_IDS_EXTRA_PROP: Lazy<ExtraProp> = Lazy::new(|| ExtraProp {
    name: "calendarVideosIds".to_owned(),
    is_required: false,
    options: vec![],
    options_limit: OptionsLimit(1),
});
pub static TYPE_PRIORITIES: Lazy<HashMap<&'static str, i32>> = Lazy::new(|| {
    vec![
        ("all", 5),
        ("movie", 4),
        ("series", 3),
        ("channel", 2),
        ("tv", 1),
        ("other", i32::MIN),
    ]
    .into_iter()
    .collect()
});
