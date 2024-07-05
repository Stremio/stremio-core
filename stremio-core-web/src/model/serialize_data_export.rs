use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::CtxError;
use stremio_core::models::data_export::DataExport;
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataExport<'a> {
        pub export_url: Option<&'a Loadable<Url, CtxError>>,
    }
}

pub fn serialize_data_export(data_export: &DataExport) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::DataExport {
        export_url: data_export
            .export_url
            .as_ref()
            .map(|(_auth_key, loadable)| loadable),
    })
    .expect("JsValue from model::DataExport")
}
