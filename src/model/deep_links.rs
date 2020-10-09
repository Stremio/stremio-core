use flate2::write::ZlibEncoder;
use flate2::Compression;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use std::io::Write;
use stremio_core::models::common::ResourceLoadable;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::library::LibraryItem;
use stremio_core::types::resource::{MetaItem, MetaItemPreview, Stream, Video};
use url::form_urlencoded;

#[derive(Serialize)]
pub struct LibraryItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
}

impl From<&LibraryItem> for LibraryItemDeepLinks {
    fn from(item: &LibraryItem) -> Self {
        LibraryItemDeepLinks {
            meta_details_videos: match &item.behavior_hints.default_video_id {
                None => Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                    utf8_percent_encode(&item.id, NON_ALPHANUMERIC)
                )),
                _ => None,
            },
            meta_details_streams: item
                .state
                .video_id
                .as_ref()
                .or_else(|| item.behavior_hints.default_video_id.as_ref())
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                        utf8_percent_encode(&item.id, NON_ALPHANUMERIC),
                        utf8_percent_encode(&video_id, NON_ALPHANUMERIC)
                    )
                }),
            player: None, // TODO use StreamsBucket
        }
    }
}

#[derive(Serialize)]
pub struct MetaItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
}

impl From<&MetaItemPreview> for MetaItemDeepLinks {
    fn from(item: &MetaItemPreview) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: match &item.behavior_hints.default_video_id {
                None => Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                    utf8_percent_encode(&item.id, NON_ALPHANUMERIC)
                )),
                _ => None,
            },
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                        utf8_percent_encode(&item.id, NON_ALPHANUMERIC),
                        utf8_percent_encode(&video_id, NON_ALPHANUMERIC)
                    )
                }),
        }
    }
}

impl From<&MetaItem> for MetaItemDeepLinks {
    fn from(item: &MetaItem) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: match &item.behavior_hints.default_video_id {
                None => Some(format!(
                    "#/metadetails/{}/{}",
                    utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                    utf8_percent_encode(&item.id, NON_ALPHANUMERIC)
                )),
                _ => None,
            },
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "#/metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.type_name, NON_ALPHANUMERIC),
                        utf8_percent_encode(&item.id, NON_ALPHANUMERIC),
                        utf8_percent_encode(&video_id, NON_ALPHANUMERIC)
                    )
                }),
        }
    }
}

#[derive(Serialize)]
pub struct VideoDeepLinks {
    pub meta_details_streams: String,
    pub player: Option<String>,
}

impl From<(&Video, &ResourceRequest)> for VideoDeepLinks {
    fn from((video, request): (&Video, &ResourceRequest)) -> Self {
        VideoDeepLinks {
            meta_details_streams: format!(
                "#/metadetails/{}/{}/{}",
                utf8_percent_encode(&request.path.type_name, NON_ALPHANUMERIC),
                utf8_percent_encode(&request.path.id, NON_ALPHANUMERIC),
                utf8_percent_encode(&video.id, NON_ALPHANUMERIC)
            ),
            player: video.streams.first().and_then(|stream| {
                if video.streams.len() == 1 {
                    Some(format!(
                        "#/player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(
                            &gz_encode(serde_json::to_string(stream).unwrap()),
                            NON_ALPHANUMERIC
                        ),
                        utf8_percent_encode(&request.base.as_str(), NON_ALPHANUMERIC),
                        utf8_percent_encode(&request.base.as_str(), NON_ALPHANUMERIC),
                        utf8_percent_encode(&request.path.type_name, NON_ALPHANUMERIC),
                        utf8_percent_encode(&request.path.id, NON_ALPHANUMERIC),
                        utf8_percent_encode(&video.id, NON_ALPHANUMERIC)
                    ))
                } else {
                    None
                }
            }),
        }
    }
}

#[derive(Serialize)]
pub struct StreamDeepLinks {
    pub player: String,
}

impl From<&Stream> for StreamDeepLinks {
    fn from(stream: &Stream) -> Self {
        StreamDeepLinks {
            player: format!(
                "#/player/{}",
                utf8_percent_encode(
                    &gz_encode(serde_json::to_string(stream).unwrap()),
                    NON_ALPHANUMERIC
                ),
            ),
        }
    }
}

impl From<(&Stream, &ResourceRequest, &ResourceRequest)> for StreamDeepLinks {
    fn from(
        (stream, stream_request, meta_request): (&Stream, &ResourceRequest, &ResourceRequest),
    ) -> Self {
        StreamDeepLinks {
            player: format!(
                "#/player/{}/{}/{}/{}/{}/{}",
                utf8_percent_encode(
                    &gz_encode(serde_json::to_string(stream).unwrap()),
                    NON_ALPHANUMERIC
                ),
                utf8_percent_encode(&stream_request.base.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(&meta_request.base.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(&meta_request.path.type_name, NON_ALPHANUMERIC),
                utf8_percent_encode(&meta_request.path.id, NON_ALPHANUMERIC),
                utf8_percent_encode(&stream_request.path.id, NON_ALPHANUMERIC)
            ),
        }
    }
}

#[derive(Serialize)]
pub struct MetaCatalogResourceDeepLinks {
    pub discover: String,
}

impl From<&ResourceLoadable<Vec<MetaItemPreview>>> for MetaCatalogResourceDeepLinks {
    fn from(catalog_resource: &ResourceLoadable<Vec<MetaItemPreview>>) -> Self {
        MetaCatalogResourceDeepLinks {
            discover: format!(
                "#/discover/{}/{}/{}?{}",
                utf8_percent_encode(&catalog_resource.request.base.as_str(), NON_ALPHANUMERIC),
                utf8_percent_encode(&catalog_resource.request.path.type_name, NON_ALPHANUMERIC),
                utf8_percent_encode(&catalog_resource.request.path.id, NON_ALPHANUMERIC),
                serde_urlencoded::to_string(&catalog_resource.request.path.extra).unwrap()
            ),
        }
    }
}

fn gz_encode(value: String) -> String {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(value.as_bytes())
        .expect("gz encode failed");
    base64::encode_config(
        encoder.finish().expect("gz encode failed"),
        base64::URL_SAFE,
    )
}
