use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::Serialize;
use stremio_core::{
    deep_links::{MetaItemDeepLinks, VideoDeepLinks},
    types::resource::{MetaItemPreview, VideoEpgInfo},
};

use crate::model::DeepLinksExt;

pub mod model {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Show<'a> {
        pub id: &'a String,
        pub title: &'a String,
        pub released: &'a Option<DateTime<Utc>>,
        pub overview: &'a Option<String>,
        pub thumbnail: &'a Option<String>,
        #[serde(flatten)]
        pub epg_info: &'a Option<VideoEpgInfo>,
        pub deep_links: VideoDeepLinks,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item<'a> {
        pub channel: &'a MetaItemPreview,
        pub deep_links: MetaItemDeepLinks,
        pub shows: Vec<Show<'a>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LiveTvContinueWatching<'a> {
        pub items: Vec<Item<'a>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_live_tv_continue_watching(
    live_tv_continue_watching: &stremio_core::models::live_tv_continue_watching::LiveTvContinueWatching,
    streaming_server_url: Option<&url::Url>,
    settings: &stremio_core::types::profile::Settings,
) -> wasm_bindgen::JsValue {
    use gloo_utils::format::JsValueSerdeExt;

    <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(&live_tv_continue_watching_model(
        live_tv_continue_watching,
        streaming_server_url,
        settings,
    ))
    .expect("JsValue from model::LiveTvContinueWatching")
}

pub fn live_tv_continue_watching_model<'a>(
    live_tv_continue_watching: &'a stremio_core::models::live_tv_continue_watching::LiveTvContinueWatching,
    streaming_server_url: Option<&url::Url>,
    settings: &stremio_core::types::profile::Settings,
) -> model::LiveTvContinueWatching<'a> {
    let streaming_server_url = streaming_server_url.cloned();

    model::LiveTvContinueWatching {
        items: live_tv_continue_watching
            .items
            .iter()
            .map(|item| model::Item {
                channel: &item.channel,
                deep_links: MetaItemDeepLinks::from((&item.channel, &item.request))
                    .into_web_deep_links(),
                shows: item
                    .shows
                    .iter()
                    .map(|show| model::Show {
                        id: &show.id,
                        title: &show.title,
                        released: &show.released,
                        overview: &show.overview,
                        thumbnail: &show.thumbnail,
                        epg_info: &show.epg_info,
                        deep_links: VideoDeepLinks::from((
                            show,
                            &item.request,
                            &streaming_server_url,
                            settings,
                        ))
                        .into_web_deep_links(),
                    })
                    .collect_vec(),
            })
            .collect_vec(),
    }
}

#[cfg(test)]
mod tests {
    use stremio_core::{
        models::live_tv_continue_watching::{Item, LiveTvContinueWatching},
        types::{
            addon::{ResourcePath, ResourceRequest},
            resource::MetaItem,
        },
    };
    use url::Url;

    use super::live_tv_continue_watching_model;

    #[test]
    fn live_tv_continue_watching_web_state() {
        let channel: MetaItem = serde_json::from_value(serde_json::json!({
            "id": "pure:axn",
            "type": "tv",
            "name": "AXN",
            "logo": "https://addon.example.com/logos/axn.png",
            "poster": "https://addon.example.com/logos/axn.png",
            "posterShape": "landscape",
            "videos": [{
                "id": "pure:axn:1m9x4p",
                "title": "Spy x Family",
                "thumbnail": "https://addon.example.com/thumbs/spy-x-family.jpg",
                "released": "2026-07-02T11:55:00Z",
                "startTime": "2026-07-02T11:55:00Z",
                "endTime": "2026-07-02T12:23:00Z",
                "streams": [{
                    "name": "PureTV",
                    "url": "https://cdn.example.com/axn/master.m3u8",
                    "behaviorHints": { "notWebReady": true }
                }]
            }]
        }))
        .unwrap();

        let request = ResourceRequest {
            base: Url::parse("https://addon.example.com/manifest.json").unwrap(),
            path: ResourcePath::without_extra("meta", "tv", "pure:axn"),
        };
        let state = LiveTvContinueWatching {
            items: vec![Item {
                channel: channel.preview.clone(),
                request,
                shows: channel.videos.clone(),
            }],
            ..Default::default()
        };

        let value = serde_json::to_value(live_tv_continue_watching_model(
            &state,
            None,
            &Default::default(),
        ))
        .unwrap();

        assert_eq!(value["items"][0]["channel"]["id"], "pure:axn");
        assert!(
            value["items"][0]["deepLinks"]["metaDetailsVideos"]
                .as_str()
                .unwrap()
                .starts_with("#/detail/tv/pure%3Aaxn"),
            "the channel card links to its meta page"
        );
        assert!(
            value["items"][0]["shows"][0]["startTime"].is_string()
                && value["items"][0]["shows"][0]["endTime"].is_string(),
            "the frontend derives the live/current show from the show times"
        );
        assert!(
            value["items"][0]["shows"][0].get("streams").is_none()
                && value["items"][0]["shows"][0].get("isLive").is_none(),
            "shows are slimmed down like the guide payload"
        );
        assert!(value["items"][0]["shows"][0]["deepLinks"]["player"].is_string());
    }
}
