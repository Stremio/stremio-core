use crate::constants::URI_COMPONENT_ENCODE_SET;
use crate::models::installed_addons_with_filters::InstalledAddonsRequest;
use crate::models::library_with_filters::LibraryRequest;
use crate::types::addon::{ExtraValue, ResourceRequest};
use crate::types::library::LibraryItem;
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use itertools::Itertools;
use percent_encoding::utf8_percent_encode;
use serde::Serialize;
use std::borrow::{Borrow, Cow};
use std::io;
use std::io::Write;
use url::form_urlencoded;

#[derive(Serialize)]
pub struct ExternalPlayerLink {
    pub href: String,
    pub download: Option<String>,
}

impl From<&Stream> for ExternalPlayerLink {
    fn from(stream: &Stream) -> Self {
        match &stream.source {
            StreamSource::Url { url } if url.scheme() == "magnet" => ExternalPlayerLink {
                href: url.as_str().to_owned(),
                download: None,
            },
            StreamSource::Url { url } => ExternalPlayerLink {
                href: format!(
                    "data:application/octet-stream;charset=utf-8;base64,{}",
                    base64::encode(format!("#EXTM3U\n#EXTINF:0\n{}", url))
                ),
                download: Some("playlist.m3u".to_owned()),
            },
            StreamSource::Torrent { .. } => ExternalPlayerLink {
                href: stream
                    .magnet_url()
                    .map(|magnet_url| magnet_url.to_string())
                    .expect("Failed to build magnet url for torrent"),
                download: None,
            },
            StreamSource::External { external_url } => ExternalPlayerLink {
                href: external_url.as_str().to_owned(),
                download: None,
            },
            StreamSource::YouTube { yt_id } => ExternalPlayerLink {
                href: format!(
                    "https://www.youtube.com/watch?v={}",
                    utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET)
                ),
                download: None,
            },
            StreamSource::PlayerFrame { player_frame_url } => ExternalPlayerLink {
                href: player_frame_url.as_str().to_owned(),
                download: None,
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
    pub external_player: Option<ExternalPlayerLink>,
}

impl From<&LibraryItem> for LibraryItemDeepLinks {
    fn from(item: &LibraryItem) -> Self {
        LibraryItemDeepLinks {
            meta_details_videos: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .cloned()
                .xor(Some(format!(
                    "stremio:///metadetails/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .state
                .video_id
                .as_ref()
                .or(item.behavior_hints.default_video_id.as_ref())
                .map(|video_id| {
                    format!(
                        "stremio:///metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(video_id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
            player: None,          // TODO use StreamsBucket
            external_player: None, // TODO use StreamsBucket
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
}

impl From<(&MetaItemPreview, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItemPreview, &ResourceRequest)) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .cloned()
                .xor(Some(format!(
                    "stremio:///metadetails/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "stremio:///metadetails/{}/{}/{}",
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(video_id, URI_COMPONENT_ENCODE_SET)
                    )
                }),
            player: item
                .behavior_hints
                .default_video_id
                .as_deref()
                .and_then(|default_video_id| {
                    Stream::youtube(default_video_id).map(|stream| (default_video_id, stream))
                })
                .map(|(default_video_id, stream)| {
                    format!(
                        "stremio:///player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(
                            &base64::encode(
                                gz_encode(serde_json::to_string(&stream).unwrap()).unwrap()
                            ),
                            URI_COMPONENT_ENCODE_SET
                        ),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(default_video_id, URI_COMPONENT_ENCODE_SET),
                    )
                }),
        }
    }
}

impl From<(&MetaItem, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItem, &ResourceRequest)) -> Self {
        MetaItemDeepLinks::from((&item.preview, request))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDeepLinks {
    pub meta_details_streams: String,
    pub player: Option<String>,
    pub external_player: Option<ExternalPlayerLink>,
}

impl From<(&Video, &ResourceRequest)> for VideoDeepLinks {
    fn from((video, request): (&Video, &ResourceRequest)) -> Self {
        let stream = video
            .streams
            .iter()
            .exactly_one()
            .ok()
            .map(Cow::Borrowed)
            .or_else(|| {
                if video.streams.is_empty() {
                    Stream::youtube(&video.id).map(Cow::Owned)
                } else {
                    None
                }
            });
        VideoDeepLinks {
            meta_details_streams: format!(
                "stremio:///metadetails/{}/{}/{}",
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
            ),
            player: stream.as_ref().map(|stream| {
                format!(
                    "stremio:///player/{}/{}/{}/{}/{}/{}",
                    utf8_percent_encode(
                        &base64::encode(
                            gz_encode(serde_json::to_string(stream.as_ref()).unwrap()).unwrap()
                        ),
                        URI_COMPONENT_ENCODE_SET
                    ),
                    utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
                )
            }),
            external_player: stream
                .as_ref()
                .map(|stream| ExternalPlayerLink::from(stream.as_ref())),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDeepLinks {
    pub player: String,
    pub external_player: ExternalPlayerLink,
}

impl From<&Stream> for StreamDeepLinks {
    fn from(stream: &Stream) -> Self {
        StreamDeepLinks {
            player: format!(
                "stremio:///player/{}",
                utf8_percent_encode(
                    &base64::encode(gz_encode(serde_json::to_string(stream).unwrap()).unwrap()),
                    URI_COMPONENT_ENCODE_SET
                ),
            ),
            external_player: ExternalPlayerLink::from(stream),
        }
    }
}

impl From<(&Stream, &ResourceRequest, &ResourceRequest)> for StreamDeepLinks {
    fn from(
        (stream, stream_request, meta_request): (&Stream, &ResourceRequest, &ResourceRequest),
    ) -> Self {
        StreamDeepLinks {
            player: format!(
                "stremio:///player/{}/{}/{}/{}/{}/{}",
                utf8_percent_encode(
                    &base64::encode(gz_encode(serde_json::to_string(stream).unwrap()).unwrap()),
                    URI_COMPONENT_ENCODE_SET
                ),
                utf8_percent_encode(stream_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(meta_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&stream_request.path.id, URI_COMPONENT_ENCODE_SET)
            ),
            external_player: ExternalPlayerLink::from(stream),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverDeepLinks {
    pub discover: String,
}

impl From<&ResourceRequest> for DiscoverDeepLinks {
    fn from(request: &ResourceRequest) -> Self {
        DiscoverDeepLinks {
            discover: format!(
                "stremio:///discover/{}/{}/{}?{}",
                utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                query_params_encode(
                    request
                        .path
                        .extra
                        .iter()
                        .map(|ExtraValue { name, value }| (name, value))
                )
            ),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonsDeepLinks {
    pub addons: String,
}

impl From<&ResourceRequest> for AddonsDeepLinks {
    fn from(request: &ResourceRequest) -> Self {
        AddonsDeepLinks {
            addons: format!(
                "stremio:///addons/{}/{}/{}",
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
            ),
        }
    }
}

impl From<&InstalledAddonsRequest> for AddonsDeepLinks {
    fn from(request: &InstalledAddonsRequest) -> Self {
        AddonsDeepLinks {
            addons: request
                .r#type
                .as_ref()
                .map(|r#type| {
                    format!(
                        "stremio:///addons/{}",
                        utf8_percent_encode(r#type, URI_COMPONENT_ENCODE_SET)
                    )
                })
                .unwrap_or_else(|| "stremio:///addons".to_owned()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDeepLinks {
    pub library: String,
}

impl From<&String> for LibraryDeepLinks {
    fn from(root: &String) -> Self {
        LibraryDeepLinks {
            library: format!("stremio:///{}", root),
        }
    }
}

impl From<(&String, &LibraryRequest)> for LibraryDeepLinks {
    fn from((root, request): (&String, &LibraryRequest)) -> Self {
        LibraryDeepLinks {
            library: match &request.r#type {
                Some(r#type) => format!(
                    "stremio:///{}/{}?{}",
                    root,
                    utf8_percent_encode(r#type, URI_COMPONENT_ENCODE_SET),
                    query_params_encode(&[
                        (
                            "sort",
                            serde_json::to_value(&request.sort)
                                .unwrap()
                                .as_str()
                                .unwrap()
                        ),
                        ("page", &request.page.to_string())
                    ]),
                ),
                _ => format!(
                    "stremio:///{}?{}",
                    root,
                    query_params_encode(&[
                        (
                            "sort",
                            serde_json::to_value(&request.sort)
                                .unwrap()
                                .as_str()
                                .unwrap()
                        ),
                        ("page", &request.page.to_string())
                    ]),
                ),
            },
        }
    }
}

fn gz_encode(value: String) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
    encoder.write_all(value.as_bytes())?;
    encoder.finish()
}

fn query_params_encode<I, K, V>(query_params: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query_params)
        .finish()
}
