use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use stremio_core::deep_links::MetaItemDeepLinks;
use stremio_core::models::common::Loadable;
use stremio_core::models::streaming_server::{PlaybackDevice, Selected, StreamingServer};
use stremio_core::runtime::EnvError;
use stremio_core::types::addon::ResourcePath;
use stremio_core::types::streaming_server::{DeviceInfo, NetworkInfo, Settings, Statistics};
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    type TorrentLoadable<'a> = Loadable<(&'a ResourcePath, MetaItemDeepLinks), &'a EnvError>;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StreamingServer<'a> {
        pub selected: &'a Selected,
        pub settings: &'a Loadable<Settings, EnvError>,
        pub base_url: &'a Option<Url>,
        pub remote_url: &'a Option<Url>,
        pub playback_devices: &'a Loadable<Vec<PlaybackDevice>, EnvError>,
        pub network_info: &'a Loadable<NetworkInfo, EnvError>,
        pub device_info: &'a Loadable<DeviceInfo, EnvError>,
        pub torrent: Option<(&'a String, TorrentLoadable<'a>)>,
        pub statistics: Option<&'a Loadable<Statistics, EnvError>>,
    }
}

pub fn serialize_streaming_server(streaming_server: &StreamingServer) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::StreamingServer {
        selected: &streaming_server.selected,
        settings: &streaming_server.settings,
        base_url: &streaming_server.base_url,
        remote_url: &streaming_server.remote_url,
        playback_devices: &streaming_server.playback_devices,
        network_info: &streaming_server.network_info,
        device_info: &streaming_server.device_info,
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
        statistics: streaming_server.statistics.as_ref(),
    })
    .expect("JsValue from model::StreamingServer")
}
