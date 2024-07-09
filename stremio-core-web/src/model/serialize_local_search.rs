use gloo_utils::format::JsValueSerdeExt;
use itertools::Itertools;
use serde::Serialize;
use wasm_bindgen::JsValue;

use stremio_core::deep_links::LocalSearchItemDeepLinks;
use stremio_core::models::local_search::{LocalSearch, Searchable};

use crate::model::deep_links_ext::DeepLinksExt;

mod model {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocalSearch<'a> {
        /// The results of the search autocompletion
        pub items: Vec<LocalSearchItem<'a>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocalSearchItem<'a> {
        pub query: &'a String,
        pub deep_links: LocalSearchItemDeepLinks,
    }
}

pub fn serialize_local_search(local_search: &LocalSearch) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::LocalSearch {
        items: local_search
            .search_results
            .to_owned()
            .iter()
            .map(|Searchable { name, .. }| model::LocalSearchItem {
                query: name,
                deep_links: LocalSearchItemDeepLinks::from(name).into_web_deep_links(),
            })
            .unique_by(|i| i.query)
            .collect(),
    })
    .expect("JsValue from model::LocalSearch")
}
