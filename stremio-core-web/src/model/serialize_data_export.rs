#[cfg(feature = "wasm")]
use {serde::Serialize, serde_wasm_bindgen::Serializer, wasm_bindgen::JsValue};

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
    model::DataExport {
        export_url: data_export
            .export_url
            .as_ref()
            .map(|(_auth_key, loadable)| loadable),
    }
    .serialize(&Serializer::json_compatible())
    .expect("JsValue from model::DataExport")
}
