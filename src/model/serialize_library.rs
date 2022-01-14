use serde::Serialize;
use stremio_core::models::library_with_filters::{LibraryWithFilters, Selected, Sort};
use stremio_core::types::resource::PosterShape;
use stremio_deeplinks::{LibraryDeepLinks, LibraryItemDeepLinks};
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItem<'a> {
        #[serde(rename = "_id")]
        pub id: &'a String,
        pub name: &'a String,
        pub r#type: &'a String,
        pub poster: &'a Option<String>,
        pub poster_shape: &'a PosterShape,
        pub progress: f64,
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
    #[serde(rename_all = "camelCase")]
    pub struct SelectablePage {
        pub deep_links: LibraryDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub sorts: Vec<SelectableSort<'a>>,
        pub prev_page: Option<SelectablePage>,
        pub next_page: Option<SelectablePage>,
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
            prev_page: library.selectable.prev_page.as_ref().map(|prev_page| {
                model::SelectablePage {
                    deep_links: LibraryDeepLinks::from((&root, &prev_page.request)),
                }
            }),
            next_page: library.selectable.next_page.as_ref().map(|next_page| {
                model::SelectablePage {
                    deep_links: LibraryDeepLinks::from((&root, &next_page.request)),
                }
            }),
        },
        catalog: library
            .catalog
            .iter()
            .map(|library_item| model::LibraryItem {
                id: &library_item.id,
                name: &library_item.name,
                r#type: &library_item.r#type,
                poster: &library_item.poster,
                poster_shape: if library_item.poster_shape == PosterShape::Landscape {
                    &PosterShape::Square
                } else {
                    &library_item.poster_shape
                },
                progress: if library_item.state.time_offset > 0 && library_item.state.duration > 0 {
                    library_item.state.time_offset as f64 / library_item.state.duration as f64
                } else {
                    0.0
                },
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect(),
    })
    .unwrap()
}
