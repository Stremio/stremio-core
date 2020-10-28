use crate::env::WebEnv;
use crate::model::deep_links::VideoDeepLinks;
use semver::Version;
use serde::Serialize;
use stremio_core::models::common::{Loadable, ResourceLoadable};
use stremio_core::models::ctx::Ctx;
use stremio_core::models::player::{Player, Selected};
use stremio_core::runtime::Env;
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
        pub id: &'a String,
        pub name: &'a String,
        pub version: &'a Version,
        pub description: &'a Option<String>,
        pub logo: &'a Option<String>,
        pub background: &'a Option<String>,
        pub types: &'a Vec<String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        pub manifest: ManifestPreview<'a>,
        pub transport_url: &'a Url,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Video<'a> {
        #[serde(flatten)]
        pub video: &'a stremio_core::types::resource::Video,
        pub upcomming: bool,
        pub watched: bool,
        pub progress: Option<u32>,
        pub scheduled: bool,
        pub deep_links: VideoDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaItem<'a> {
        #[serde(flatten)]
        pub meta_item: &'a stremio_core::types::resource::MetaItem,
        pub videos: Vec<Video<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Subtitles<'a> {
        #[serde(flatten)]
        pub subtitles: &'a stremio_core::types::resource::Subtitles,
        pub origin: &'a String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItemState<'a> {
        pub time_offset: &'a u64,
        #[serde(rename = "video_id")]
        pub video_id: &'a Option<String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItem<'a> {
        #[serde(rename = "_id")]
        pub id: &'a String,
        pub state: LibraryItemState<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Player<'a> {
        pub selected: &'a Option<Selected>,
        pub meta_item: Option<model::MetaItem<'a>>,
        pub subtitles: Vec<model::Subtitles<'a>>,
        pub next_video_deep_links: Option<VideoDeepLinks>,
        pub library_item: Option<LibraryItem<'a>>,
        pub title: Option<String>,
        pub addon: Option<model::DescriptorPreview<'a>>,
    }
}

pub fn serialize_player(player: &Player, ctx: &Ctx<WebEnv>) -> JsValue {
    JsValue::from_serde(&model::Player {
        selected: &player.selected,
        meta_item: player
            .meta_item
            .as_ref()
            .and_then(|meta_item| match meta_item {
                ResourceLoadable {
                    request,
                    content: Loadable::Ready(meta_item),
                } => Some((request, meta_item)),
                _ => None,
            })
            .map(|(request, meta_item)| model::MetaItem {
                meta_item,
                videos: meta_item
                    .videos
                    .iter()
                    .map(|video| model::Video {
                        video,
                        upcomming: meta_item.behavior_hints.has_scheduled_videos
                            && meta_item
                                .released
                                .map(|released| released > WebEnv::now())
                                .unwrap_or(true),
                        watched: false, // TODO use library
                        progress: None, // TODO use library,
                        scheduled: meta_item.behavior_hints.has_scheduled_videos,
                        deep_links: VideoDeepLinks::from((video, request)),
                    })
                    .collect(),
            }),
        subtitles: player
            .subtitles
            .iter()
            .filter_map(|subtitles| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == subtitles.request.base)
                    .map(|addon| (addon, subtitles))
            })
            .filter_map(|(addon, subtitles)| match subtitles {
                ResourceLoadable {
                    content: Loadable::Ready(subtitles),
                    ..
                } => Some((addon, subtitles)),
                _ => None,
            })
            .flat_map(|(addon, subtitles)| {
                subtitles.iter().map(move |subtitles| (addon, subtitles))
            })
            .map(|(addon, subtitles)| model::Subtitles {
                subtitles,
                origin: &addon.manifest.name,
            })
            .collect(),
        next_video_deep_links: player
            .selected
            .as_ref()
            .and_then(|selected| selected.meta_request.as_ref())
            .zip(player.next_video.as_ref())
            .map(|(meta_request, next_video)| VideoDeepLinks::from((next_video, meta_request))),
        library_item: player
            .library_item
            .as_ref()
            .map(|library_item| model::LibraryItem {
                id: &library_item.id,
                state: model::LibraryItemState {
                    time_offset: &library_item.state.time_offset,
                    video_id: &library_item.state.video_id,
                },
            }),
        title: player.selected.as_ref().and_then(|selected| {
            player
                .meta_item
                .as_ref()
                .and_then(|meta_item| match meta_item {
                    ResourceLoadable {
                        content: Loadable::Ready(meta_item),
                        ..
                    } => Some(meta_item),
                    _ => None,
                })
                .zip(selected.stream_request.as_ref())
                .map(|(meta_item, stream_request)| {
                    match meta_item
                        .videos
                        .iter()
                        .find(|video| video.id == stream_request.path.id)
                    {
                        Some(video) if meta_item.behavior_hints.default_video_id.is_none() => {
                            match &video.series_info {
                                Some(series_info) => format!(
                                    "{} - {} ({}x{})",
                                    &meta_item.name,
                                    &video.title,
                                    &series_info.season,
                                    &series_info.episode
                                ),
                                _ => format!("{} - {}", &meta_item.name, &video.title),
                            }
                        }
                        _ => meta_item.name.to_owned(),
                    }
                })
                .or_else(|| selected.stream.title.to_owned())
        }),
        addon: player
            .selected
            .as_ref()
            .and_then(|selected| selected.stream_request.as_ref())
            .and_then(|stream_request| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == stream_request.base)
            })
            .map(|addon| model::DescriptorPreview {
                transport_url: &addon.transport_url,
                manifest: model::ManifestPreview {
                    id: &addon.manifest.id,
                    name: &addon.manifest.name,
                    version: &addon.manifest.version,
                    description: &addon.manifest.description,
                    logo: &addon.manifest.logo,
                    background: &addon.manifest.background,
                    types: &addon.manifest.types,
                },
            }),
    })
    .unwrap()
}
