use crate::types::addon::Descriptor;
use lazy_static::lazy_static;
use std::collections::HashMap;
use url::Url;

pub const SCHEMA_VERSION_STORAGE_KEY: &str = "schema_version";
pub const PROFILE_STORAGE_KEY: &str = "profile";
pub const LIBRARY_STORAGE_KEY: &str = "library";
pub const LIBRARY_RECENT_STORAGE_KEY: &str = "library_recent";
pub const LIBRARY_COLLECTION_NAME: &str = "libraryItem";
pub const SKIP_EXTRA_NAME: &str = "skip";
pub const SEARCH_EXTRA_NAME: &str = "search";
pub const META_RESOURCE_NAME: &str = "meta";
pub const STREAM_RESOURCE_NAME: &str = "stream";
pub const SUBTITLES_RESOURCE_NAME: &str = "subtitles";
pub const ADDON_MANIFEST_PATH: &str = "/manifest.json";
pub const ADDON_LEGACY_PATH: &str = "/stremio/v1";
pub const CATALOG_PAGE_SIZE: usize = 100;
pub const CATALOG_PREVIEW_SIZE: usize = 10;
pub const LIBRARY_RECENT_COUNT: usize = 200;
pub const WATCHED_THRESHOLD_COEF: f64 = 0.7;
pub const SCHEMA_VERSION: u32 = 2;

lazy_static! {
    pub static ref API_URL: Url = Url::parse("https://api.strem.io").expect("API_URL parse failed");
    pub static ref STREAMING_SERVER_URL: Url =
        Url::parse("http://127.0.0.1:11470").expect("STREAMING_SERVER_URL parse failed");
    pub static ref OFFICIAL_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(stremio_official_addons::ADDONS)
            .expect("OFFICIAL_ADDONS parse failed");
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
