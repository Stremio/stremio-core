use serde::Serialize;
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::CtxError;
use stremio_core::models::data_export::DataExport;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataExport<'a> {
        pub export_url: Option<Loadable<String, &'a CtxError>>,
    }
}

pub fn serialize_data_export(data_export: &DataExport) -> JsValue {
    JsValue::from_serde(&model::DataExport {
        export_url: data_export
            .export_url
            .as_ref()
            .map(|(_auth_key, loadable)| {
                let loadable = match loadable {
                    Loadable::Ready(url) => Loadable::Ready(
                        url.as_str().to_string()
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error),
                };
                loadable
            }),
    })
    .unwrap()
}
