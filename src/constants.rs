use crate::types::addon::Descriptor;
use lazy_static::lazy_static;
use url::Url;

pub const PROFILE_STORAGE_KEY: &str = "profile";
pub const LIBRARY_STORAGE_KEY: &str = "library";
pub const LIBRARY_RECENT_STORAGE_KEY: &str = "recent_library";
pub const LIBRARY_COLLECTION_NAME: &str = "libraryItem";
pub const SKIP_EXTRA_NAME: &str = "skip";
pub const SEARCH_EXTRA_NAME: &str = "search";
pub const META_RESOURCE_NAME: &str = "meta";
pub const STREAM_RESOURCE_NAME: &str = "stream";
pub const SUBTITLES_RESOURCE_NAME: &str = "subtitles";
pub const CATALOG_PAGE_SIZE: usize = 100;
pub const CATALOG_PREVIEW_SIZE: usize = 10;
pub const LIBRARY_RECENT_COUNT: usize = 200;
pub const WATCHED_THRESHOLD_COEF: f64 = 0.7;

lazy_static! {
    pub static ref API_URL: Url =
        Url::parse("https://api.strem.io").expect("API_URL parse cannot fail");
    pub static ref STREAMING_SERVER_URL: Url =
        Url::parse("http://127.0.0.1:11470").expect("STREAMING_SERVER_URL parse cannot fail");
    pub static ref OFFICIAL_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}
