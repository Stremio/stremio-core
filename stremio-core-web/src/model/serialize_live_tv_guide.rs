use chrono::NaiveDate;
use gloo_utils::format::JsValueSerdeExt;
use itertools::Itertools;
use serde::Serialize;
use stremio_core::{
    constants::META_RESOURCE_NAME,
    deep_links::{LiveTvGuideDeepLinks, MetaItemDeepLinks, VideoDeepLinks},
    models::{
        common::{Loadable, ResourceError},
        live_tv_guide::Selected,
    },
    runtime::Env,
    types::{
        addon::{ResourcePath, ResourceRequest},
        resource::{MetaItemPreview, Video},
    },
};
use wasm_bindgen::JsValue;

use crate::{env::WebEnv, model::DeepLinksExt};

mod model {
    use super::*;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Show<'a> {
        #[serde(flatten)]
        pub video: &'a Video,
        pub is_live: bool,
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
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LiveTvGuide<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Option<Loadable<(), &'a ResourceError>>,
        pub channels: Vec<ChannelGuide<'a>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_live_tv_guide(
    live_tv_guide: &stremio_core::models::live_tv_guide::LiveTvGuide,
    streaming_server_url: Option<&url::Url>,
    settings: &stremio_core::types::profile::Settings,
) -> JsValue {
    let now = WebEnv::now();
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

    <JsValue as JsValueSerdeExt>::from_serde(&model::LiveTvGuide {
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
        },
        catalog: live_tv_guide.catalog.as_ref().and_then(|catalog| {
            catalog
                .content
                .as_ref()
                .map(|content| content.as_ref().map(|_| ()))
        }),
        channels: live_tv_guide
            .channels
            .iter()
            .filter_map(|channel_guide| {
                // channels are derived from the loaded catalog, so its
                // request is always available here; deep links of channels
                // and their shows are built against the channel's meta
                // resource on the guide catalog's addon
                let catalog_request = live_tv_guide.catalog.as_ref()?;
                let meta_request = ResourceRequest {
                    base: catalog_request.request.base.to_owned(),
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
                            video: show,
                            is_live: show
                                .epg_info
                                .as_ref()
                                .is_some_and(|epg_info| epg_info.is_live(now)),
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
    })
    .expect("JsValue from model::LiveTvGuide")
}
