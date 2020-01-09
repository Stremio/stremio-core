use crate::types::addons::Descriptor;
use lazy_static::lazy_static;

pub const API_URL: &str = "https://api.strem.io";
pub const USER_DATA_KEY: &str = "userData";
pub const SKIP_EXTRA_NAME: &str = "skip";
pub const SEARCH_EXTRA_NAME: &str = "search";
pub const META_RESOURCE_NAME: &str = "meta";
pub const STREAM_RESOURCE_NAME: &str = "stream";
pub const SUBTITLES_RESOURCE_NAME: &str = "subtitles";
pub const META_CATALOG_PAGE_SIZE: usize = 100;
pub const META_CATALOG_PREVIEW_SIZE: usize = 10;

lazy_static! {
    pub static ref OFFICIAL_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}
