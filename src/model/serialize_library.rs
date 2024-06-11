use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use stremio_core::deep_links::{LibraryDeepLinks, LibraryItemDeepLinks};
use stremio_core::models::ctx::Ctx;
use stremio_core::models::library_with_filters::{LibraryWithFilters, Selected, Sort};
use stremio_core::types::resource::PosterShape;
use stremio_core::types::streams::StreamsItemKey;
use url::Url;
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
        pub poster: &'a Option<Url>,
        pub poster_shape: &'a PosterShape,
        pub notifications: usize,
        pub progress: f64,
        pub watched: bool,
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
        pub next_page: bool,
    }
    #[derive(Serialize)]
    pub struct LibraryWithFilters<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Vec<LibraryItem<'a>>,
    }
}

pub fn serialize_library<F>(
    library: &LibraryWithFilters<F>,
    ctx: &Ctx,
    streaming_server_url: Option<&Url>,
    root: String,
) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::LibraryWithFilters {
        selected: &library.selected,
        selectable: model::Selectable {
            types: library
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_type.request))
                        .into_web_deep_links(),
                })
                .collect(),
            sorts: library
                .selectable
                .sorts
                .iter()
                .map(|selectable_sort| model::SelectableSort {
                    sort: &selectable_sort.sort,
                    selected: &selectable_sort.selected,
                    deep_links: LibraryDeepLinks::from((&root, &selectable_sort.request))
                        .into_web_deep_links(),
                })
                .collect(),
            next_page: library.selectable.next_page.is_some(),
        },
        catalog: library
            .catalog
            .iter()
            .map(|library_item| {
                // Try to get the stream from the StreamBucket
                // given that we have a video_id in the LibraryItemState!
                let streams_item = library_item.state.video_id.as_ref().and_then(|video_id| {
                    ctx.streams.items.get(&StreamsItemKey {
                        meta_id: library_item.id.to_owned(),
                        video_id: video_id.to_owned(),
                    })
                });

                model::LibraryItem {
                    id: &library_item.id,
                    name: &library_item.name,
                    r#type: &library_item.r#type,
                    poster: &library_item.poster,
                    poster_shape: if library_item.poster_shape == PosterShape::Landscape {
                        &PosterShape::Square
                    } else {
                        &library_item.poster_shape
                    },
                    notifications: ctx
                        .notifications
                        .items
                        .get(&library_item.id)
                        .map_or(0, |item| item.len()),
                    progress: library_item.progress(),
                    watched: library_item.watched(),
                    deep_links: LibraryItemDeepLinks::from((
                        library_item,
                        streams_item,
                        streaming_server_url,
                        &ctx.profile.settings,
                    ))
                    .into_web_deep_links(),
                }
            })
            .collect(),
    })
    .expect("JsValue from model::LibraryWithFilters")
}
