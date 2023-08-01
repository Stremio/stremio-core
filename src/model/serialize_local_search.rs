use serde::Serialize;
use stremio_core::models::local_search::LocalSearch;
use wasm_bindgen::JsValue;

mod model {
    use stremio_core::{
        models::{common::Loadable, local_search::Searchable},
        runtime::EnvError,
    };

    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocalSearch {
        pub current_records: Vec<Searchable>,
        /// The results of the search autocompletion
        pub search_results: Vec<Searchable>,
        pub latest_records: Loadable<Vec<Searchable>, EnvError>,
    }
}

pub fn serialize_local_search(local_search: &LocalSearch) -> JsValue {
    JsValue::from_serde(&model::LocalSearch {
        current_records: local_search.current_records.to_owned(),
        search_results: local_search.search_results.to_owned(),
        latest_records: local_search.latest_records.to_owned(),
    })
    .unwrap()
}
