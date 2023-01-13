mod error_link;

use crate::constants::URI_COMPONENT_ENCODE_SET;
use crate::deep_links::error_link::ErrorLink;
use crate::models::installed_addons_with_filters::InstalledAddonsRequest;
use crate::models::library_with_filters::LibraryRequest;
use crate::types::addon::{ExtraValue, ResourcePath, ResourceRequest};
use crate::types::library::LibraryItem;
use crate::types::query_params_encode;
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video};
use percent_encoding::utf8_percent_encode;
use serde::Serialize;
use url::Url;

#[derive(Default, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerLink {
    pub href: Option<String>,
    pub download: Option<String>,
    pub streaming_server: Option<String>,
    pub android_tv: Option<String>,
    pub tizen: Option<String>,
    pub webos: Option<String>,
    pub file_name: Option<String>,
}

impl From<(&Stream, &Url)> for ExternalPlayerLink {
    fn from((stream, streaming_server_url): (&Stream, &Url)) -> Self {
        match &stream.source {
            StreamSource::Url { url } if url.scheme() == "magnet" => ExternalPlayerLink {
                href: Some(url.as_str().to_owned()),
                download: Some(url.as_str().to_owned()),
                ..Default::default()
            },
            StreamSource::Url { url } => ExternalPlayerLink {
                href: Some(format!(
                    "data:application/octet-stream;charset=utf-8;base64,{}",
                    base64::encode(format!("#EXTM3U\n#EXTINF:0\n{url}"))
                )),
                download: Some(url.as_str().to_owned()),
                file_name: Some("playlist.m3u".to_owned()),
                ..Default::default()
            },
            StreamSource::Torrent {
                info_hash,
                file_idx,
                announce,
            } => ExternalPlayerLink {
                href: Some(
                    stream
                        .magnet_url()
                        .map(|magnet_url| magnet_url.to_string())
                        .expect("Failed to build magnet url for torrent"),
                ),
                download: Some(
                    stream
                        .magnet_url()
                        .map(|magnet_url| magnet_url.to_string())
                        .expect("Failed to build magnet url for torrent"),
                ),
                streaming_server: streaming_server_url
                    .join(&format!("{}/", hex::encode(info_hash)))
                    .map(|url| match file_idx {
                        Some(idx) => url.join(&format!("{}/", idx)),
                        None => Ok(url),
                    })
                    .map(|url| match url {
                        Ok(url) => url.join(&format!("?tr={}", announce.join("&tr="))),
                        _ => url,
                    })
                    .map(|url| match url {
                        Ok(url) => Some(url.to_string()),
                        _ => None,
                    })
                    .expect("Failed to build streaming server url for torrent"),
                file_name: Some("torrent".to_string()),
                ..Default::default()
            },
            StreamSource::External {
                external_url,
                android_tv_url,
                tizen_url,
                webos_url,
            } => ExternalPlayerLink {
                href: external_url.as_ref().map(|url| url.as_str().to_owned()),
                download: external_url.as_ref().map(|url| url.as_str().to_owned()),
                android_tv: android_tv_url.as_ref().map(|url| url.as_str().to_owned()),
                tizen: tizen_url.to_owned(),
                webos: webos_url.to_owned(),
                ..Default::default()
            },
            StreamSource::YouTube { yt_id } => ExternalPlayerLink {
                href: Some(format!(
                    "data:application/octet-stream;charset=utf-8;base64,{}",
                    base64::encode(format!(
                        "#EXTM3U\n#EXTINF:0\n{}yt/{}",
                        streaming_server_url, yt_id
                    ))
                )),
                download: Some(format!(
                    "https://www.youtube.com/watch?v={}",
                    utf8_percent_encode(yt_id, URI_COMPONENT_ENCODE_SET)
                )),
                streaming_server: Some(format!("{}yt/{}", streaming_server_url, yt_id)),
                ..Default::default()
            },
            StreamSource::PlayerFrame { player_frame_url } => ExternalPlayerLink {
                href: Some(player_frame_url.as_str().to_owned()),
                download: Some(player_frame_url.as_str().to_owned()),
                ..Default::default()
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
                    "stremio:///detail/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .state
                .video_id
                .as_ref()
                .filter(|_video_id| item.state.time_offset > 0)
                .or(item.behavior_hints.default_video_id.as_ref())
                .map(|video_id| {
                    format!(
                        "stremio:///detail/{}/{}/{}",
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

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
}

impl From<&ResourcePath> for MetaItemDeepLinks {
    fn from(resource_path: &ResourcePath) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: Some(format!(
                "stremio:///detail/{}/{}",
                utf8_percent_encode(&resource_path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&resource_path.id, URI_COMPONENT_ENCODE_SET)
            )),
            meta_details_streams: None,
            player: None,
        }
    }
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
                    "stremio:///detail/{}/{}",
                    utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET)
                ))),
            meta_details_streams: item
                .behavior_hints
                .default_video_id
                .as_ref()
                .map(|video_id| {
                    format!(
                        "stremio:///detail/{}/{}/{}",
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
                    Ok::<_, anyhow::Error>(format!(
                        "stremio:///player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(&stream.encode()?, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&item.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(default_video_id, URI_COMPONENT_ENCODE_SET),
                    ))
                })
                .transpose()
                .unwrap_or_else(|error| Some(ErrorLink::from(error).into())),
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

impl From<(&Video, &ResourceRequest, &Url)> for VideoDeepLinks {
    fn from((video, request, streaming_server_url): (&Video, &ResourceRequest, &Url)) -> Self {
        let stream = video.stream();
        VideoDeepLinks {
            meta_details_streams: format!(
                "stremio:///detail/{}/{}/{}",
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
            ),
            player: stream
                .as_ref()
                .map(|stream| {
                    Ok::<_, anyhow::Error>(format!(
                        "stremio:///player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(&stream.encode()?, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
                    ))
                })
                .transpose()
                .unwrap_or_else(|error| Some(ErrorLink::from(error).into())),
            external_player: stream
                .as_ref()
                .map(|stream| ExternalPlayerLink::from((stream.as_ref(), streaming_server_url))),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDeepLinks {
    pub player: String,
    pub external_player: ExternalPlayerLink,
}

impl From<(&Stream, &Url)> for StreamDeepLinks {
    fn from((stream, streaming_server_url): (&Stream, &Url)) -> Self {
        StreamDeepLinks {
            player: stream
                .encode()
                .map(|stream| {
                    format!(
                        "stremio:///player/{}",
                        utf8_percent_encode(&stream, URI_COMPONENT_ENCODE_SET),
                    )
                })
                .unwrap_or_else(|error| ErrorLink::from(error).into()),
            external_player: ExternalPlayerLink::from((stream, streaming_server_url)),
        }
    }
}

impl From<(&Stream, &ResourceRequest, &ResourceRequest, &Url)> for StreamDeepLinks {
    fn from(
        (stream, stream_request, meta_request, streaming_server_url): (
            &Stream,
            &ResourceRequest,
            &ResourceRequest,
            &Url,
        ),
    ) -> Self {
        StreamDeepLinks {
            player: stream
                .encode()
                .map(|stream| {
                    format!(
                        "stremio:///player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(&stream, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(stream_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(meta_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&stream_request.path.id, URI_COMPONENT_ENCODE_SET)
                    )
                })
                .unwrap_or_else(|error| ErrorLink::from(error).into()),
            external_player: ExternalPlayerLink::from((stream, streaming_server_url)),
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
            library: format!("stremio:///{root}"),
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
