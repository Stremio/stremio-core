use crate::model::deep_links::{LibraryDeepLinks, LibraryItemDeepLinks};
use serde::Serialize;
use stremio_core::models::library_with_filters::{LibraryWithFilters, Selected, Sort};
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
    pub struct SelectableType<'a> {
        pub r#type: &'a Option<String>,
        pub selected: &'a bool,
        pub deep_links: LibraryDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableSort<'a> {
        pub sort: &'a Sort,
        pub selected: &'a bool,
        pub deep_links: LibraryDeepLinks,
    }
    #[derive(Serialize)]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub sorts: Vec<SelectableSort<'a>>,
    }
    #[derive(Serialize)]
    pub struct LibraryWithFilters<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Vec<LibraryItem<'a>>,
    }
}

pub fn serialize_library<F>(library: &LibraryWithFilters<F>, root: String) -> JsValue {
    JsValue::from_serde(&model::LibraryWithFilters {
        selected: &library.selected,
        selectable: model::Selectable {
            types: library
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_type.request)),
                })
                .collect(),
            sorts: library
                .selectable
                .sorts
                .iter()
                .map(|selectable_sort| model::SelectableSort {
                    sort: &selectable_sort.sort,
                    selected: &selectable_sort.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_sort.request)),
                })
                .collect(),
        },
        catalog: library
            .catalog
            .iter()
            .map(|library_item| model::LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect(),
    })
    .unwrap()
}
