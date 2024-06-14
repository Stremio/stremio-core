use crate::env::WebEnv;
use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use semver::Version;
use serde::Serialize;
use stremio_core::deep_links::{StreamDeepLinks, VideoDeepLinks};
use stremio_core::models::common::{Loadable, ResourceError, ResourceLoadable};
use stremio_core::models::ctx::Ctx;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::Env;
use stremio_core::types::{
    addon::{ResourcePath, ResourceRequest},
    streams::StreamItemState,
};
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Stream<'a> {
        #[serde(flatten)]
        pub stream: &'a stremio_core::types::resource::Stream,
        pub deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
        pub id: &'a String,
        pub name: &'a String,
        pub version: &'a Version,
        pub description: &'a Option<String>,
        pub logo: &'a Option<Url>,
        pub background: &'a Option<Url>,
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
        pub upcoming: bool,
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
        pub id: String,
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
    pub struct Selected<'a> {
        pub stream: Stream<'a>,
        pub stream_request: &'a Option<ResourceRequest>,
        pub meta_request: &'a Option<ResourceRequest>,
        pub subtitles_path: &'a Option<ResourcePath>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Player<'a> {
        pub selected: Option<Selected<'a>>,
        pub meta_item: Option<Loadable<model::MetaItem<'a>, &'a ResourceError>>,
        pub subtitles: Vec<model::Subtitles<'a>>,
        pub next_video: Option<Video<'a>>,
        pub series_info: Option<&'a stremio_core::types::resource::SeriesInfo>,
        pub library_item: Option<LibraryItem<'a>>,
        pub stream_state: Option<&'a StreamItemState>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub intro_outro: Option<&'a stremio_core::types::player::IntroOutro>,
        pub title: Option<String>,
        pub addon: Option<model::DescriptorPreview<'a>>,
    }
}

pub fn serialize_player(player: &Player, ctx: &Ctx, streaming_server: &StreamingServer) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::Player {
        selected: player.selected.as_ref().map(|selected| model::Selected {
            stream: model::Stream {
                stream: &selected.stream,
                deep_links: StreamDeepLinks::from((
                    &selected.stream,
                    &streaming_server.base_url,
                    &ctx.profile.settings,
                ))
                .into_web_deep_links(),
            },
            stream_request: &selected.stream_request,
            meta_request: &selected.meta_request,
            subtitles_path: &selected.subtitles_path,
        }),
        meta_item: player
            .meta_item
            .as_ref()
            .map(|ResourceLoadable { request, content }| match &content {
                Some(Loadable::Loading) | None => Loadable::Loading,
                Some(Loadable::Err(error)) => Loadable::Err(error),
                Some(Loadable::Ready(meta_item)) => {
                    Loadable::Ready(model::MetaItem {
                        meta_item,
                        videos: meta_item
                            .videos
                            .iter()
                            .map(|video| model::Video {
                                video,
                                upcoming: meta_item.preview.behavior_hints.has_scheduled_videos
                                    && video.released > Some(WebEnv::now()),
                                watched: false, // TODO use library
                                progress: None, // TODO use library,
                                scheduled: meta_item.preview.behavior_hints.has_scheduled_videos,
                                deep_links: VideoDeepLinks::from((
                                    video,
                                    request,
                                    &streaming_server.base_url,
                                    &ctx.profile.settings,
                                ))
                                .into_web_deep_links(),
                            })
                            .collect(),
                    })
                }
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
                    content: Some(Loadable::Ready(subtitles)),
                    ..
                } => Some((addon, subtitles)),
                _ => None,
            })
            .flat_map(|(addon, subtitles)| {
                subtitles
                    .iter()
                    .enumerate()
                    .map(move |(position, subtitles)| model::Subtitles {
                        subtitles,
                        id: format!("{}_{}", addon.transport_url, position),
                        origin: &addon.manifest.name,
                    })
            })
            .collect(),
        next_video: player
            .selected
            .as_ref()
            .and_then(|selected| {
                selected
                    .meta_request
                    .as_ref()
                    .zip(selected.stream_request.as_ref())
            })
            .zip(player.next_video.as_ref())
            .map(|((meta_request, stream_request), video)| model::Video {
                video,
                upcoming: player
                    .meta_item
                    .as_ref()
                    .and_then(|meta_item| match meta_item {
                        ResourceLoadable {
                            content: Some(Loadable::Ready(meta_item)),
                            ..
                        } => Some(meta_item),
                        _ => None,
                    })
                    .map(|meta_item| {
                        meta_item.preview.behavior_hints.has_scheduled_videos
                            && video.released > Some(WebEnv::now())
                    })
                    .unwrap_or_default(),
                watched: false, // TODO use library
                progress: None, // TODO use library,
                scheduled: player
                    .meta_item
                    .as_ref()
                    .and_then(|meta_item| match meta_item {
                        ResourceLoadable {
                            content: Some(Loadable::Ready(meta_item)),
                            ..
                        } => Some(meta_item.preview.behavior_hints.has_scheduled_videos),
                        _ => None,
                    })
                    .unwrap_or_default(),
                deep_links: VideoDeepLinks::from((
                    video,
                    stream_request,
                    meta_request,
                    &streaming_server.base_url,
                    &ctx.profile.settings,
                ))
                .into_web_deep_links(),
            }),
        series_info: player.series_info.as_ref(),
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
        stream_state: player.stream_state.as_ref(),
        intro_outro: player.intro_outro.as_ref(),
        title: player.selected.as_ref().and_then(|selected| {
            player
                .meta_item
                .as_ref()
                .and_then(|meta_item| match meta_item {
                    ResourceLoadable {
                        content: Some(Loadable::Ready(meta_item)),
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
                        Some(video)
                            if meta_item.preview.behavior_hints.default_video_id.is_none() =>
                        {
                            match &video.series_info {
                                Some(series_info) => format!(
                                    "{} - {} ({}x{})",
                                    &meta_item.preview.name,
                                    &video.title,
                                    &series_info.season,
                                    &series_info.episode
                                ),
                                _ => format!("{} - {}", &meta_item.preview.name, &video.title),
                            }
                        }
                        _ => meta_item.preview.name.to_owned(),
                    }
                })
                .or_else(|| selected.stream.name.to_owned())
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
    .expect("JsValue from model::Player")
}
