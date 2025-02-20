#[cfg(feature = "wasm")]
use {serde_wasm_bindgen, wasm_bindgen::JsValue};

pub use model::*;

mod model {
    use serde::Serialize;
    use url::Url;

    use stremio_core::models::{common::Loadable, ctx::CtxError};
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataExport<'a> {
        pub export_url: Option<&'a Loadable<Url, CtxError>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_data_export(
    data_export: &stremio_core::models::data_export::DataExport,
) -> JsValue {
    serde_wasm_bindgen::to_value(&model::DataExport {
        export_url: data_export
            .export_url
            .as_ref()
            .map(|(_auth_key, loadable)| loadable),
    })
    .expect("JsValue from model::DataExport")
}
