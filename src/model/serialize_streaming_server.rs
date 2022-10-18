use crate::model::deep_links_ext::DeepLinksExt;
use serde::Serialize;
use stremio_core::deep_links::MetaItemDeepLinks;
use stremio_core::models::common::Loadable;
use stremio_core::models::streaming_server::{Selected, Settings, StreamingServer};
use stremio_core::runtime::EnvError;
use stremio_core::types::addon::ResourcePath;
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StreamingServer<'a> {
        pub selected: &'a Selected,
        pub settings: &'a Loadable<Settings, EnvError>,
        pub base_url: &'a Loadable<Url, EnvError>,
        pub torrent: Option<(
            &'a String,
            Loadable<(&'a ResourcePath, MetaItemDeepLinks), &'a EnvError>,
        )>,
    }
}

pub fn serialize_streaming_server(streaming_server: &StreamingServer) -> JsValue {
    JsValue::from_serde(&model::StreamingServer {
        selected: &streaming_server.selected,
        settings: &streaming_server.settings,
        base_url: &streaming_server.base_url,
        torrent: streaming_server
            .torrent
            .as_ref()
            .map(|(info_hash, loadable)| {
                let loadable = match loadable {
                    Loadable::Ready(resource_path) => Loadable::Ready((
                        resource_path,
                        MetaItemDeepLinks::from(resource_path).into_web_deep_links(),
                    )),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error),
                };
                (info_hash, loadable)
            }),
    })
    .unwrap()
}
