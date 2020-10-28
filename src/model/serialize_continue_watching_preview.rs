use crate::model::deep_links::{LibraryDeepLinks, LibraryItemDeepLinks};
use serde::Serialize;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItem<'a> {
        #[serde(flatten)]
        pub library_item: &'a stremio_core::types::library::LibraryItem,
        pub deep_links: LibraryItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinueWatchingPreview<'a> {
        pub library_items: Vec<LibraryItem<'a>>,
        pub deep_links: LibraryDeepLinks,
    }
}

pub fn serialize_continue_watching_preview(
    continue_watching_preview: &ContinueWatchingPreview,
) -> JsValue {
    JsValue::from_serde(&model::ContinueWatchingPreview {
        library_items: continue_watching_preview
            .library_items
            .iter()
            .map(|library_item| model::LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect::<Vec<_>>(),
        deep_links: LibraryDeepLinks::from(&"continuewatching".to_owned()),
    })
    .unwrap()
}
