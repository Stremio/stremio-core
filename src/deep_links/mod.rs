use percent_encoding::utf8_percent_encode;
use regex::Regex;
use serde::Serialize;
use url::Url;

use crate::{
    constants::URI_COMPONENT_ENCODE_SET,
    models::{
        calendar::YearMonthDate, installed_addons_with_filters::InstalledAddonsRequest,
        library_with_filters::LibraryRequest,
    },
    types::{
        addon::{ExtraValue, ResourcePath, ResourceRequest},
        library::LibraryItem,
        profile::Settings,
        query_params_encode,
        resource::{MetaItem, MetaItemPreview, Stream, StreamSource, Video},
        streams::StreamsItem,
    },
};

pub use error_link::ErrorLink;

mod error_link;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// VisionOS
    pub visionos: Option<String>,
}

#[derive(Default, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerLink {
    pub download: Option<String>,
    pub streaming: Option<String>,
    /// m3u data URI
    pub playlist: Option<String>,
    /// Filename of the playlist
    pub file_name: Option<String>,
    pub open_player: Option<OpenPlayerLink>,
    /// External URL for Web
    pub web: Option<Url>,
    /// External URI for Android TV
    pub android_tv: Option<Url>,
    /// External payload for Tizen
    pub tizen: Option<String>,
    /// External payload for webOS
    pub webos: Option<String>,
}

/// Using `&Option<Url>` is not encouraged, use `.as_ref()` to get an `Option<&Url>` instead!
impl From<(&Stream, &Option<Url>, &Settings)> for ExternalPlayerLink {
    fn from((stream, streaming_server_url, settings): (&Stream, &Option<Url>, &Settings)) -> Self {
        Self::from((stream, streaming_server_url.as_ref(), settings))
    }
}

impl From<(&Stream, Option<&Url>, &Settings)> for ExternalPlayerLink {
    /// Create an [`ExternalPlayerLink`] using the [`Stream`],
    /// the server url (from [`StreamingServer::base_url`] which indicates a running or not server)
    /// and the user's [`Settings`] in order to use the [`Settings::player_type`] for generating a
    /// player-specific url.
    ///
    /// [`StreamingServer::base_url`]: crate::models::streaming_server::StreamingServer::base_url
    fn from((stream, streaming_server_url, settings): (&Stream, Option<&Url>, &Settings)) -> Self {
        // Use streaming_server_url from settings if streaming_server is reachable
        let streaming_server_url = streaming_server_url.map(|_| &settings.streaming_server_url);
        let http_regex = Regex::new(r"https?://").unwrap();
        let download = stream.download_url();
        let streaming = stream.streaming_url(streaming_server_url);
        let playlist = stream.m3u_data_uri(streaming_server_url);
        let file_name = playlist.as_ref().map(|_| "playlist.m3u".to_owned());
        let open_player = match &streaming {
            Some(url) => match settings.player_type.as_ref() {
                Some(player_type) => match player_type.as_str() {
                    "choose" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;type=video/any;scheme=https;end",
                            http_regex.replace(url.as_str(), "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "vlc" => Some(OpenPlayerLink {
                        ios: Some(format!("vlc-x-callback://x-callback-url/stream?url={url}")),
                        visionos: Some(format!("vlc-x-callback://x-callback-url/stream?url={url}")),
                        android: Some(format!(
                            "{}#Intent;package=org.videolan.vlc;type=video;scheme=https;end",
                            http_regex.replace(url.as_str(), "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "mxplayer" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;package=com.mxtech.videoplayer.ad;type=video;scheme=https;end",
                            http_regex.replace(url.as_str(), "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "justplayer" => Some(OpenPlayerLink {
                        android: Some(format!(
                            "{}#Intent;package=com.brouken.player;type=video;scheme=https;end",
                            http_regex.replace(url.as_str(), "intent://"),
                        )),
                        ..Default::default()
                    }),
                    "outplayer" => Some(OpenPlayerLink {
                        ios: Some(http_regex.replace(url.as_str(), "outplayer://").to_string()),
                        visionos: Some(http_regex.replace(url.as_str(), "outplayer://").to_string()),
                        ..Default::default()
                    }),
                    "infuse" => Some(OpenPlayerLink {
                        ios: Some(format!("infuse://x-callback-url/play?url={url}")),
                       ..Default::default()
                    }),
                    "iina" => Some(OpenPlayerLink {
                        macos: Some(format!("iina://weblink?url={url}")),
                       ..Default::default()
                    }),
                    "mpv" => Some(OpenPlayerLink {
                        macos: Some(format!("mpv://{url}")),
                       ..Default::default()
                    }),
                    "moonplayer" => Some(OpenPlayerLink {
                        visionos: Some(format!("moonplayer://open?url={url}")),
                        ..Default::default()
                    }),
                    "m3u" => Some(OpenPlayerLink {
                        linux: playlist.to_owned(),
                        windows: playlist.to_owned(),
                        macos: playlist.to_owned(),
                        android: playlist.to_owned(),
                        ios: playlist.to_owned(),
                       ..Default::default()
                    }),
                    _ => None,
                },
                None => None,
            },
            None => None,
        };
        let (web, android_tv, tizen, webos) = match &stream.source {
            StreamSource::External {
                external_url,
                android_tv_url,
                tizen_url,
                webos_url,
                ..
            } => (
                external_url.to_owned(),
                android_tv_url.to_owned(),
                tizen_url.to_owned(),
                webos_url.to_owned(),
            ),
            _ => (None, None, None, None),
        };
        ExternalPlayerLink {
            download,
            streaming: streaming.as_ref().map(ToString::to_string),
            playlist,
            file_name,
            open_player,
            web,
            android_tv,
            tizen,
            webos,
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

impl From<(&LibraryItem, Option<&StreamsItem>, Option<&Url>, &Settings)> for LibraryItemDeepLinks {
    fn from(
        (item, streams_item, streaming_server_url, settings): (
            &LibraryItem,
            Option<&StreamsItem>,
            Option<&Url>,
            &Settings,
        ),
    ) -> Self {
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
            // We have the stream so use the same logic as in StreamDeepLinks
            player: streams_item.map(|streams_item| match streams_item.stream.encode() {
                Ok(encoded_stream) => format!(
                    "stremio:///player/{}/{}/{}/{}/{}/{}",
                    utf8_percent_encode(&encoded_stream, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(
                        streams_item.stream_transport_url.as_str(),
                        URI_COMPONENT_ENCODE_SET
                    ),
                    utf8_percent_encode(
                        streams_item.meta_transport_url.as_str(),
                        URI_COMPONENT_ENCODE_SET
                    ),
                    utf8_percent_encode(&streams_item.r#type, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&streams_item.meta_id, URI_COMPONENT_ENCODE_SET),
                    utf8_percent_encode(&streams_item.video_id, URI_COMPONENT_ENCODE_SET)
                ),
                Err(error) => ErrorLink::from(error).into(),
            }),
            // We have the streams bucket item so use the same logic as in StreamDeepLinks
            external_player: streams_item.map(|item| {
                ExternalPlayerLink::from((&item.stream, streaming_server_url, settings))
            }),
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

impl From<(MetaItemPreview, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (MetaItemPreview, &ResourceRequest)) -> Self {
        Self::from((&item, request))
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
    pub meta_details_videos: String,
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
            meta_details_videos: format!(
                "stremio:///detail/{}/{}",
                utf8_percent_encode(&request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&request.path.id, URI_COMPONENT_ENCODE_SET),
            ),
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

impl
    From<(
        &Video,
        &ResourceRequest,
        &ResourceRequest,
        &Option<Url>,
        &Settings,
    )> for VideoDeepLinks
{
    fn from(
        (video, stream_request, meta_request, streaming_server_url, settings): (
            &Video,
            &ResourceRequest,
            &ResourceRequest,
            &Option<Url>,
            &Settings,
        ),
    ) -> Self {
        let stream = video.stream();
        VideoDeepLinks {
            meta_details_videos: format!(
                "stremio:///detail/{}/{}",
                utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
            ),
            meta_details_streams: format!(
                "stremio:///detail/{}/{}/{}",
                utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
            ),
            player: stream
                .as_ref()
                .map(|stream| {
                    Ok::<_, anyhow::Error>(format!(
                        "stremio:///player/{}/{}/{}/{}/{}/{}",
                        utf8_percent_encode(&stream.encode()?, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(stream_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(meta_request.base.as_str(), URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&meta_request.path.r#type, URI_COMPONENT_ENCODE_SET),
                        utf8_percent_encode(&meta_request.path.id, URI_COMPONENT_ENCODE_SET),
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
    /// Create a [`StreamDeepLinks`] using the [`Stream`],
    /// the server url (from [`StreamingServer::base_url`] which indicates a running or not server)
    /// and the user's [`Settings`] in order to use the [`Settings::player_type`] for generating a
    /// player-specific url.
    ///
    /// [`StreamingServer::base_url`]: crate::models::streaming_server::StreamingServer::base_url
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
    /// Create a [`StreamDeepLinks`] using the [`Stream`], stream request, meta request,
    /// the server url (from [`StreamingServer::base_url`] which indicates a running or not server)
    /// and the user's [`Settings`] in order to use the [`Settings::player_type`] for generating a
    /// player-specific url.
    ///
    /// [`StreamingServer::base_url`]: crate::models::streaming_server::StreamingServer::base_url
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
                    query_params_encode(&[(
                        "sort",
                        serde_json::to_value(&request.sort)
                            .unwrap()
                            .as_str()
                            .unwrap()
                    )]),
                ),
                _ => format!(
                    "stremio:///{}?{}",
                    root,
                    query_params_encode(&[(
                        "sort",
                        serde_json::to_value(&request.sort)
                            .unwrap()
                            .as_str()
                            .unwrap()
                    )]),
                ),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDeepLinks {
    pub calendar: String,
}

impl From<&YearMonthDate> for CalendarDeepLinks {
    fn from(date: &YearMonthDate) -> Self {
        Self {
            calendar: format!("stremio:///calendar/{}/{}", date.year, date.month),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarItemDeepLinks {
    pub meta_details_streams: String,
}

impl From<(&MetaItem, &Video)> for CalendarItemDeepLinks {
    fn from((meta_item, video): (&MetaItem, &Video)) -> Self {
        CalendarItemDeepLinks {
            meta_details_streams: format!(
                "stremio:///detail/{}/{}/{}",
                utf8_percent_encode(&meta_item.preview.r#type, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&meta_item.preview.id, URI_COMPONENT_ENCODE_SET),
                utf8_percent_encode(&video.id, URI_COMPONENT_ENCODE_SET)
            ),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryItemDeepLinks {
    pub search: String,
}

impl From<&String> for SearchHistoryItemDeepLinks {
    fn from(query: &String) -> Self {
        SearchHistoryItemDeepLinks {
            search: format!(
                "stremio:///search?query={}",
                utf8_percent_encode(query, URI_COMPONENT_ENCODE_SET),
            ),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSearchItemDeepLinks {
    pub search: String,
}

impl From<&String> for LocalSearchItemDeepLinks {
    fn from(query: &String) -> Self {
        LocalSearchItemDeepLinks {
            search: format!(
                "stremio:///search?query={}",
                utf8_percent_encode(query, URI_COMPONENT_ENCODE_SET),
            ),
        }
    }
}
