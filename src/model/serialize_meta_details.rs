use crate::env::WebEnv;
use crate::model::deep_links_ext::DeepLinksExt;
use either::Either;
use itertools::Itertools;
use serde::Serialize;
use std::iter;
use stremio_core::constants::META_RESOURCE_NAME;
use stremio_core::deep_links::{MetaItemDeepLinks, StreamDeepLinks, VideoDeepLinks};
use stremio_core::models::common::{Loadable, ResourceError, ResourceLoadable};
use stremio_core::models::ctx::Ctx;
use stremio_core::models::meta_details::{MetaDetails, Selected as MetaDetailsSelected};
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
        pub logo: &'a Option<String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        pub manifest: ManifestPreview<'a>,
        pub transport_url: &'a Url,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Stream<'a> {
        #[serde(flatten)]
        pub stream: &'a stremio_core::types::resource::Stream,
        pub deep_links: StreamDeepLinks,
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
        pub trailer_streams: Vec<Stream<'a>>,
        pub in_library: bool,
        pub watched: bool,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a, T> {
        pub content: Loadable<T, &'a ResourceError>,
        pub addon: DescriptorPreview<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaExtension<'a> {
        pub url: &'a Url,
        pub name: &'a String,
        pub addon: DescriptorPreview<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaDetails<'a> {
        pub selected: &'a Option<MetaDetailsSelected>,
        pub meta_item: Option<ResourceLoadable<'a, MetaItem<'a>>>,
        pub streams: Vec<ResourceLoadable<'a, Vec<Stream<'a>>>>,
        pub meta_extensions: Vec<MetaExtension<'a>>,
        pub title: Option<String>,
    }
}

pub fn serialize_meta_details(meta_details: &MetaDetails, ctx: &Ctx) -> JsValue {
    let meta_item = meta_details
        .meta_items
        .iter()
        .find(|meta_item| matches!(&meta_item.content, Some(Loadable::Ready(_))))
        .or_else(|| {
            if meta_details
                .meta_items
                .iter()
                .all(|meta_item| matches!(&meta_item.content, Some(Loadable::Err(_))))
            {
                meta_details.meta_items.first()
            } else {
                meta_details
                    .meta_items
                    .iter()
                    .find(|meta_item| matches!(&meta_item.content, Some(Loadable::Loading)))
            }
        });
    let streams = if meta_details.meta_streams.is_empty() {
        meta_details.streams.iter()
    } else {
        meta_details.meta_streams.iter()
    };
    JsValue::from_serde(&model::MetaDetails {
        selected: &meta_details.selected,
        meta_item: meta_item
            .and_then(|meta_item| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == meta_item.request.base)
                    .map(|addon| (meta_item, addon))
            })
            .map(|(meta_item, addon)| model::ResourceLoadable {
                content: match &meta_item {
                    ResourceLoadable {
                        request,
                        content: Some(Loadable::Ready(meta_item)),
                    } => Loadable::Ready(model::MetaItem {
                        meta_item,
                        videos: meta_item
                            .videos
                            .iter()
                            .map(|video| model::Video {
                                video,
                                upcomming: meta_item.preview.behavior_hints.has_scheduled_videos
                                    && meta_item
                                        .preview
                                        .released
                                        .map(|released| released > WebEnv::now())
                                        .unwrap_or(true),
                                watched: meta_details
                                    .watched
                                    .as_ref()
                                    .map(|watched| watched.get_video(&video.id))
                                    .unwrap_or_default(),
                                progress: None, // TODO use library,
                                scheduled: meta_item.preview.behavior_hints.has_scheduled_videos,
                                deep_links: VideoDeepLinks::from((video, request))
                                    .into_web_deep_links(),
                            })
                            .collect::<Vec<_>>(),
                        trailer_streams: meta_item
                            .preview
                            .trailer_streams
                            .iter()
                            .map(|stream| model::Stream {
                                stream,
                                deep_links: StreamDeepLinks::from(stream).into_web_deep_links(),
                            })
                            .collect::<Vec<_>>(),
                        in_library: ctx
                            .library
                            .items
                            .get(&meta_item.preview.id)
                            .map(|library_item| !library_item.removed)
                            .unwrap_or_default(),
                        watched: ctx
                            .library
                            .items
                            .get(&meta_item.preview.id)
                            .map(|library_item| library_item.state.times_watched > 0)
                            .unwrap_or_default(),
                        deep_links: MetaItemDeepLinks::from((meta_item, request))
                            .into_web_deep_links(),
                    }),
                    ResourceLoadable {
                        content: Some(Loadable::Loading),
                        ..
                    }
                    | ResourceLoadable { content: None, .. } => Loadable::Loading,
                    ResourceLoadable {
                        content: Some(Loadable::Err(error)),
                        ..
                    } => Loadable::Err(error),
                },
                addon: model::DescriptorPreview {
                    transport_url: &addon.transport_url,
                    manifest: model::ManifestPreview {
                        id: &addon.manifest.id,
                        name: &addon.manifest.name,
                        logo: &addon.manifest.logo,
                    },
                },
            }),
        streams: streams
            .filter_map(|streams| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == streams.request.base)
                    .map(|addon| (streams, addon))
            })
            .map(|(streams, addon)| model::ResourceLoadable {
                content: match streams {
                    ResourceLoadable {
                        request,
                        content: Some(Loadable::Ready(streams)),
                    } => Loadable::Ready(
                        streams
                            .iter()
                            .map(|stream| model::Stream {
                                stream,
                                deep_links: meta_item
                                    .map_or_else(
                                        || StreamDeepLinks::from(stream),
                                        |meta_item| {
                                            StreamDeepLinks::from((
                                                stream,
                                                request,
                                                &meta_item.request,
                                            ))
                                        },
                                    )
                                    .into_web_deep_links(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    ResourceLoadable {
                        content: Some(Loadable::Loading),
                        ..
                    }
                    | ResourceLoadable { content: None, .. } => Loadable::Loading,
                    ResourceLoadable {
                        content: Some(Loadable::Err(error)),
                        ..
                    } => Loadable::Err(error),
                },
                addon: model::DescriptorPreview {
                    transport_url: &addon.transport_url,
                    manifest: model::ManifestPreview {
                        id: &addon.manifest.id,
                        name: &addon.manifest.name,
                        logo: &addon.manifest.logo,
                    },
                },
            })
            .collect::<Vec<_>>(),
        meta_extensions: meta_details
            .meta_items
            .iter()
            .filter_map(|meta_item| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == meta_item.request.base)
                    .map(|addon| (meta_item, addon))
            })
            .flat_map(|(meta_item, addon)| match meta_item {
                ResourceLoadable {
                    content: Some(Loadable::Ready(meta_item)),
                    ..
                } => Either::Left(
                    meta_item
                        .preview
                        .links
                        .iter()
                        .filter(|link| link.category == META_RESOURCE_NAME)
                        .map(move |link| (link, addon)),
                ),
                _ => Either::Right(iter::empty()),
            })
            .unique_by(|(link, _)| &link.url)
            .map(|(link, addon)| model::MetaExtension {
                url: &link.url,
                name: &link.name,
                addon: model::DescriptorPreview {
                    transport_url: &addon.transport_url,
                    manifest: model::ManifestPreview {
                        id: &addon.manifest.id,
                        name: &addon.manifest.name,
                        logo: &addon.manifest.logo,
                    },
                },
            })
            .collect::<Vec<_>>(),
        title: meta_item
            .as_ref()
            .and_then(|meta_item| match meta_item {
                ResourceLoadable {
                    content: Some(Loadable::Ready(meta_item)),
                    ..
                } => Some(meta_item),
                _ => None,
            })
            .map(|meta_item| {
                meta_details
                    .selected
                    .as_ref()
                    .and_then(|selected| selected.stream_path.as_ref())
                    .and_then(|stream_path| {
                        match meta_item
                            .videos
                            .iter()
                            .find(|video| video.id == stream_path.id)
                        {
                            Some(video)
                                if meta_item.preview.behavior_hints.default_video_id.is_none() =>
                            {
                                match &video.series_info {
                                    Some(series_info) => Some(format!(
                                        "{} - {} ({}x{})",
                                        &meta_item.preview.name,
                                        &video.title,
                                        &series_info.season,
                                        &series_info.episode
                                    )),
                                    _ => Some(format!(
                                        "{} - {}",
                                        &meta_item.preview.name, &video.title
                                    )),
                                }
                            }
                            _ => None,
                        }
                    })
                    .unwrap_or_else(|| meta_item.preview.name.to_owned())
            }),
    })
    .unwrap()
}
