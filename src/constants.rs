use crate::types::addon::{Descriptor, ExtraProp, OptionsLimit};
use lazy_static::lazy_static;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use std::collections::HashMap;
use url::Url;

pub const SCHEMA_VERSION_STORAGE_KEY: &str = "schema_version";
pub const PROFILE_STORAGE_KEY: &str = "profile";
pub const LIBRARY_STORAGE_KEY: &str = "library";
pub const LIBRARY_RECENT_STORAGE_KEY: &str = "library_recent";
pub const LIBRARY_COLLECTION_NAME: &str = "libraryItem";
pub const SEARCH_EXTRA_NAME: &str = "search";
pub const META_RESOURCE_NAME: &str = "meta";
pub const STREAM_RESOURCE_NAME: &str = "stream";
pub const CATALOG_RESOURCE_NAME: &str = "catalog";
pub const SUBTITLES_RESOURCE_NAME: &str = "subtitles";
pub const ADDON_MANIFEST_PATH: &str = "/manifest.json";
pub const ADDON_LEGACY_PATH: &str = "/stremio/v1";
pub const CATALOG_PAGE_SIZE: usize = 100;
pub const CATALOG_PREVIEW_SIZE: usize = 10;
pub const LIBRARY_RECENT_COUNT: usize = 200;
pub const WATCHED_THRESHOLD_COEF: f64 = 0.7;
pub const CREDITS_THRESHOLD_COEF: f64 = 0.9;
pub const SCHEMA_VERSION: u32 = 5;
pub const IMDB_LINK_CATEGORY: &str = "imdb";
pub const GENRES_LINK_CATEGORY: &str = "Genres";
pub const CINEMETA_TOP_CATALOG_ID: &str = "top";
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

lazy_static! {
    pub static ref CINEMETA_URL: Url = Url::parse("https://v3-cinemeta.strem.io/manifest.json")
        .expect("CINEMETA_URL parse failed");
    pub static ref API_URL: Url = Url::parse("https://api.strem.io").expect("API_URL parse failed");
    pub static ref LINK_API_URL: Url =
        Url::parse("https://link.stremio.com").expect("LINK_API_URL parse failed");
    pub static ref STREAMING_SERVER_URL: Url =
        Url::parse("http://127.0.0.1:11470").expect("STREAMING_SERVER_URL parse failed");
    pub static ref IMDB_URL: Url = Url::parse("https://imdb.com").expect("IMDB_URL parse failed");
    pub static ref OFFICIAL_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(stremio_official_addons::ADDONS)
            .expect("OFFICIAL_ADDONS parse failed");
    pub static ref SKIP_EXTRA_PROP: ExtraProp = ExtraProp {
        name: "skip".to_owned(),
        is_required: false,
        options: vec![],
        options_limit: OptionsLimit::default(),
    };
    pub static ref TYPE_PRIORITIES: HashMap<&'static str, i32> = vec![
        ("movie", 4),
        ("series", 3),
        ("channel", 2),
        ("tv", 1),
        ("other", i32::MIN)
    ]
    .into_iter()
    .collect();
}
