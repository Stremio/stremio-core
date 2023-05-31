mod error_link;

use crate::constants::URI_COMPONENT_ENCODE_SET;
use crate::deep_links::error_link::ErrorLink;
use crate::models::installed_addons_with_filters::InstalledAddonsRequest;
use crate::models::library_with_filters::LibraryRequest;
use crate::types::addon::{ExtraValue, ResourcePath, ResourceRequest};
use crate::types::library::LibraryItem;
use crate::types::profile::Settings;
use crate::types::query_params_encode;
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video};
use percent_encoding::utf8_percent_encode;
use regex::Regex;
use serde::Serialize;
use url::Url;

#[derive(Default, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenPlayerLink {
    pub ios: Option<String>,
    pub android: Option<String>,
    pub windows: Option<String>,
    pub macos: Option<String>,
    pub linux: Option<String>,
    pub tizen: Option<String>,
    pub webos: Option<String>,
    pub chromeos: Option<String>,
    pub roku: Option<String>,
}

#[derive(Default, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerLink {
    pub href: Option<String>,
    pub download: Option<String>,
    pub streaming: Option<String>,
    pub open_player: Option<OpenPlayerLink>,
    pub android_tv: Option<String>,
    pub tizen: Option<String>,
    pub webos: Option<String>,
    pub file_name: Option<String>,
}

impl From<(&Stream, &Option<Url>, &Settings)> for ExternalPlayerLink {
    fn from((stream, streaming_server_url, settings): (&Stream, &Option<Url>, &Settings)) -> Self {
        let http_regex = Regex::new(r"https?://").unwrap();
        let download = stream.download_url();
        let streaming = stream.streaming_url(streaming_server_url.as_ref());
        let m3u_uri = stream.m3u_data_uri(streaming_server_url.as_ref());
        let file_name = m3u_uri.as_ref().map(|_| "playlist.m3u".to_owned());
        let href = m3u_uri.or_else(|| download.to_owned());
        let open_player = match &streaming {
            Some(url) => match settings.player_type.as_ref() {
                Some(player_type) => match player_type.as_str() {
                    "choose" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;type=video/any;scheme=https;end",
                            http_regex.replace(url, "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "vlc" => Some(OpenPlayerLink {
                        ios: Some(format!("vlc-x-callback://x-callback-url/stream?url={url}")),
                        android: Some(format!(
                            "{}#Intent;package=org.videolan.vlc;type=video;scheme=https;end",
                            http_regex.replace(url, "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "mxplayer" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;package=com.mxtech.videoplayer.ad;type=video;scheme=https;end",
                            http_regex.replace(url, "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "justplayer" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;package=com.brouken.player;type=video;scheme=https;end",
                            http_regex.replace(url, "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "outplayer" => Some(OpenPlayerLink {
                        ios: Some(format!("{}", http_regex.replace(url, "outplayer://"))),
                        ..Default::default()
                    }),
                    "infuse" => Some(OpenPlayerLink {
                        ios: Some(format!("infuse://x-callback-url/play?url={url}")),
                       ..Default::default()
                    }),
                    _ => None,
                },
                None => None,
            },
            None => None,
        };
        let (android_tv, tizen, webos) = match &stream.source {
            StreamSource::External {
                android_tv_url,
                tizen_url,
                webos_url,
                ..
            } => (
                android_tv_url.as_ref().map(|url| url.to_string()),
                tizen_url.to_owned(),
                webos_url.to_owned(),
            ),
            _ => (None, None, None),
        };
        ExternalPlayerLink {
            href,
            download,
            streaming,
            open_player,
            android_tv,
            tizen,
            webos,
            file_name,
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

impl From<(&Video, &ResourceRequest, &Option<Url>, &Settings)> for VideoDeepLinks {
    fn from(
        (video, request, streaming_server_url, settings): (
            &Video,
            &ResourceRequest,
            &Option<Url>,
            &Settings,
        ),
    ) -> Self {
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
            external_player: stream.as_ref().map(|stream| {
                ExternalPlayerLink::from((stream.as_ref(), streaming_server_url, settings))
            }),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDeepLinks {
    pub player: String,
    pub external_player: ExternalPlayerLink,
}

impl From<(&Stream, &Option<Url>, &Settings)> for StreamDeepLinks {
    fn from((stream, streaming_server_url, settings): (&Stream, &Option<Url>, &Settings)) -> Self {
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
            external_player: ExternalPlayerLink::from((stream, streaming_server_url, settings)),
        }
    }
}

impl
    From<(
        &Stream,
        &ResourceRequest,
        &ResourceRequest,
        &Option<Url>,
        &Settings,
    )> for StreamDeepLinks
{
    fn from(
        (stream, stream_request, meta_request, streaming_server_url, settings): (
            &Stream,
            &ResourceRequest,
            &ResourceRequest,
            &Option<Url>,
            &Settings,
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
            external_player: ExternalPlayerLink::from((stream, streaming_server_url, settings)),
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
