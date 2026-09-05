use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use serde::Serialize;
use stremio_core::{
    constants::META_RESOURCE_NAME,
    deep_links::{LiveTvGuideDeepLinks, MetaItemDeepLinks, VideoDeepLinks},
    models::{
        common::{Loadable, ResourceError},
        live_tv_guide::{SelectablePage, Selected},
    },
    types::{
        addon::{ResourcePath, ResourceRequest},
        resource::{MetaItemPreview, VideoEpgInfo},
    },
};

use crate::model::DeepLinksExt;

pub mod model {
    use super::*;

    /// A slimmed down [`Video`] - no `streams`/`trailerStreams` (the guide
    /// payload is large and playback goes through the deep links) and no
    /// snapshot state like "is live" that goes stale without a re-render
    /// (the frontend derives it from `startTime`/`endTime`)
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
    pub struct ChannelGuide<'a> {
        pub channel: &'a MetaItemPreview,
        pub deep_links: MetaItemDeepLinks,
        pub shows: Vec<Show<'a>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableCatalog<'a> {
        pub catalog: &'a String,
        pub addon_name: &'a String,
        pub selected: &'a bool,
        pub deep_links: LiveTvGuideDeepLinks,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableDate<'a> {
        pub date: &'a NaiveDate,
        pub deep_links: LiveTvGuideDeepLinks,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub catalogs: Vec<SelectableCatalog<'a>>,
        pub prev_date: Option<SelectableDate<'a>>,
        pub next_date: Option<SelectableDate<'a>>,
        pub today: &'a Option<NaiveDate>,
        pub next_page: &'a Option<SelectablePage>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LiveTvGuide<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Vec<Loadable<(), &'a ResourceError>>,
        pub channels: Vec<ChannelGuide<'a>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_live_tv_guide(
    live_tv_guide: &stremio_core::models::live_tv_guide::LiveTvGuide,
    streaming_server_url: Option<&url::Url>,
    settings: &stremio_core::types::profile::Settings,
) -> wasm_bindgen::JsValue {
    use gloo_utils::format::JsValueSerdeExt;
    use stremio_core::runtime::Env;

    <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(&live_tv_guide_model(
        live_tv_guide,
        streaming_server_url,
        settings,
        crate::env::WebEnv::now(),
    ))
    .expect("JsValue from model::LiveTvGuide")
}

/// Builds the serializable web state of the [`LiveTvGuide`] model,
/// platform-agnostic so that it is testable natively
pub fn live_tv_guide_model<'a>(
    live_tv_guide: &'a stremio_core::models::live_tv_guide::LiveTvGuide,
    streaming_server_url: Option<&url::Url>,
    settings: &stremio_core::types::profile::Settings,
    now: DateTime<Utc>,
) -> model::LiveTvGuide<'a> {
    let streaming_server_url = streaming_server_url.cloned();
    let selected_date = live_tv_guide
        .selected
        .as_ref()
        .and_then(|selected| selected.date);
    let selected_request = live_tv_guide
        .selected
        .as_ref()
        .and_then(|selected| selected.request.as_ref());
    let date_deep_links = |date: &NaiveDate| match selected_request {
        Some(request) => LiveTvGuideDeepLinks::from((request, date)).into_web_deep_links(),
        None => LiveTvGuideDeepLinks::from(date).into_web_deep_links(),
    };

    model::LiveTvGuide {
        selected: &live_tv_guide.selected,
        selectable: model::Selectable {
            catalogs: live_tv_guide
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| model::SelectableCatalog {
                    catalog: &selectable_catalog.catalog,
                    addon_name: &selectable_catalog.addon_name,
                    selected: &selectable_catalog.selected,
                    deep_links: match selected_date.as_ref() {
                        Some(date) => {
                            LiveTvGuideDeepLinks::from((&selectable_catalog.request, date))
                                .into_web_deep_links()
                        }
                        None => LiveTvGuideDeepLinks::from((
                            &selectable_catalog.request,
                            &now.date_naive(),
                        ))
                        .into_web_deep_links(),
                    },
                })
                .collect_vec(),
            prev_date: live_tv_guide.selectable.prev_date.as_ref().map(|date| {
                model::SelectableDate {
                    date,
                    deep_links: date_deep_links(date),
                }
            }),
            next_date: live_tv_guide.selectable.next_date.as_ref().map(|date| {
                model::SelectableDate {
                    date,
                    deep_links: date_deep_links(date),
                }
            }),
            today: &live_tv_guide.selectable.today,
            next_page: &live_tv_guide.selectable.next_page,
        },
        catalog: live_tv_guide
            .catalog
            .iter()
            .filter_map(|page| {
                page.content
                    .as_ref()
                    .map(|content| content.as_ref().map(|_| ()))
            })
            .collect_vec(),
        channels: live_tv_guide
            .channels
            .iter()
            .filter_map(|channel_guide| {
                // channels exist only when a catalog is selected; deep links
                // of channels and their shows are built against the
                // channel's meta resource on the guide catalog's addon
                let meta_request = ResourceRequest {
                    base: selected_request?.base.to_owned(),
                    path: ResourcePath::without_extra(
                        META_RESOURCE_NAME,
                        &channel_guide.channel.r#type,
                        &channel_guide.channel.id,
                    ),
                };
                Some(model::ChannelGuide {
                    channel: &channel_guide.channel,
                    deep_links: MetaItemDeepLinks::from((&channel_guide.channel, &meta_request))
                        .into_web_deep_links(),
                    shows: channel_guide
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
                                &meta_request,
                                &streaming_server_url,
                                settings,
                            ))
                            .into_web_deep_links(),
                        })
                        .collect_vec(),
                })
            })
            .collect_vec(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use stremio_core::{
        models::{
            common::{Loadable, ResourceLoadable},
            live_tv_guide::{
                ChannelGuide, LiveTvGuide, Selectable, SelectableCatalog, SelectablePage, Selected,
            },
        },
        types::{
            addon::{ExtraValue, ResourcePath, ResourceRequest},
            resource::MetaItem,
        },
    };
    use url::Url;

    use super::live_tv_guide_model;

    #[test]
    fn live_tv_guide_web_state() {
        let channel: MetaItem = serde_json::from_value(serde_json::json!({
            "id": "pure:axn",
            "type": "tv",
            "name": "AXN",
            "logo": "https://addon.example.com/logos/axn.png",
            "poster": "https://addon.example.com/logos/axn.png",
            "posterShape": "landscape",
            "behaviorHints": { "hasScheduledVideos": true },
            "videos": [
                {
                    "id": "pure:axn:8f3k2j",
                    "title": "S.W.A.T.",
                    "overview": "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                    "thumbnail": "https://addon.example.com/thumbs/swat.jpg",
                    "released": "2026-07-02T10:30:00Z",
                    "startTime": "2026-07-02T10:30:00Z",
                    "endTime": "2026-07-02T11:55:00Z",
                    "runtime": "85 min",
                    "releaseInfo": "2018",
                    "genres": ["Ação", "Drama", "Policial"],
                    "cast": ["Shemar Moore"],
                    "directors": [],
                    "links": [],
                    "streams": [{
                        "name": "PureTV",
                        "description": "AXN",
                        "url": "https://cdn.example.com/axn/master.m3u8",
                        "behaviorHints": { "notWebReady": true }
                    }]
                },
                {
                    "id": "pure:axn:1m9x4p",
                    "title": "Spy x Family",
                    "overview": "Ut enim ad minim veniam, quis nostrud exercitation.",
                    "thumbnail": "https://addon.example.com/thumbs/spy-x-family.jpg",
                    "released": "2026-07-02T11:55:00Z",
                    "startTime": "2026-07-02T11:55:00Z",
                    "endTime": "2026-07-02T12:23:00Z",
                    "runtime": "28 min",
                    "releaseInfo": "2022",
                    "genres": ["Anime", "Comédia"],
                    "cast": ["Takuya Eguchi"],
                    "directors": [],
                    "links": [],
                    "streams": [{
                        "name": "PureTV",
                        "description": "AXN",
                        "url": "https://cdn.example.com/axn/master.m3u8",
                        "behaviorHints": { "notWebReady": true }
                    }]
                }
            ]
        }))
        .unwrap();

        let base = Url::parse("https://addon.example.com/manifest.json").unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let date_extra = ExtraValue {
            name: "date".to_owned(),
            value: "2026-07-02".to_owned(),
        };
        let catalog_request = ResourceRequest {
            base: base.clone(),
            path: ResourcePath::without_extra("catalog", "tv", "puretv-guide"),
        };
        let page_request = ResourceRequest {
            base: base.clone(),
            path: ResourcePath::with_extra("catalog", "tv", "puretv-guide", &[date_extra.clone()]),
        };
        let next_page_request = ResourceRequest {
            base: base.clone(),
            path: ResourcePath::with_extra(
                "catalog",
                "tv",
                "puretv-guide",
                &[
                    ExtraValue {
                        name: "skip".to_owned(),
                        value: "1".to_owned(),
                    },
                    date_extra,
                ],
            ),
        };

        // deserialization sorts videos by released DESC: [spy, swat];
        // the model derives shows sorted by startTime ASC: [swat, spy]
        let shows = vec![channel.videos[1].clone(), channel.videos[0].clone()];
        let state = LiveTvGuide {
            selected: Some(Selected {
                request: Some(catalog_request.clone()),
                date: Some(date),
                utc_offset: 0,
            }),
            selectable: Selectable {
                catalogs: vec![SelectableCatalog {
                    catalog: "PureTV".to_owned(),
                    addon_name: "PureTV".to_owned(),
                    selected: true,
                    request: catalog_request,
                }],
                prev_date: date.pred_opt(),
                next_date: date.succ_opt(),
                today: Some(date),
                next_page: Some(SelectablePage {
                    request: next_page_request,
                }),
            },
            catalog: vec![ResourceLoadable {
                request: page_request,
                content: Some(Loadable::Ready(vec![channel.clone()])),
            }],
            channels: vec![ChannelGuide {
                channel: channel.preview.clone(),
                shows,
            }],
        };

        // "now" is 12:00 - S.W.A.T. has ended, Spy x Family is on air
        let now = Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap();
        let value =
            serde_json::to_value(live_tv_guide_model(&state, None, &Default::default(), now))
                .unwrap();

        assert_eq!(value["channels"][0]["shows"][0]["id"], "pure:axn:8f3k2j");
        assert!(
            value["channels"][0]["shows"][0]["startTime"].is_string()
                && value["channels"][0]["shows"][0]["endTime"].is_string(),
            "the frontend derives the live state from the show times"
        );
        assert!(
            value["channels"][0]["shows"][0].get("streams").is_none()
                && value["channels"][0]["shows"][0]
                    .get("trailerStreams")
                    .is_none()
                && value["channels"][0]["shows"][0].get("isLive").is_none(),
            "shows are slimmed down to keep the guide payload small"
        );
        assert!(value["channels"][0]["shows"][1]["deepLinks"]["player"].is_string());
        assert!(value["channels"][0]["deepLinks"]["metaDetailsVideos"]
            .as_str()
            .unwrap()
            .starts_with("#/detail/tv/pure%3Aaxn"));
        assert_eq!(value["catalog"][0]["type"], "Ready");
        assert_eq!(
            value["selectable"]["nextPage"]["request"]["path"]["extra"][0][0],
            "skip"
        );
    }
}
