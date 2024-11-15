use crate::constants::{BASE64, URI_COMPONENT_ENCODE_SET};
use crate::deep_links::StreamDeepLinks;
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::profile::Settings;
use crate::types::resource::{Stream, StreamBehaviorHints, StreamProxyHeaders, StreamSource};
use base64::Engine;
use percent_encoding::utf8_percent_encode;
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

const MAGNET_STR_URL: &str = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c";
const HTTP_STR_URL: &str = "http://domain.root/some/path";
const HTTP_WITH_QUERY_STR_URL: &str = "http://domain.root/some/path?param=some&foo=bar";
const BASE64_HTTP_URL: &str = "data:application/octet-stream;charset=utf-8;base64,I0VYVE0zVQojRVhUSU5GOjAKaHR0cDovL2RvbWFpbi5yb290L3NvbWUvcGF0aA==";
const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11470";
const YT_ID: &str = "aqz-KE-bpKQ";

#[test]
fn stream_deep_links_magnet() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str(MAGNET_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(sdl.player, "stremio:///player/eAEBRgC5%2F3sidXJsIjoibWFnbmV0Oj94dD11cm46YnRpaDpkZDgyNTVlY2RjN2NhNTVmYjBiYmY4MTMyM2Q4NzA2MmRiMWY2ZDFjIn0%2BMhZF".to_string());
    assert_eq!(
        sdl.external_player.download,
        Some(MAGNET_STR_URL.to_owned()),
    );
    assert_eq!(sdl.external_player.file_name, None);
}

#[test]
fn stream_deep_links_http() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str(HTTP_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(
        &sdl.player,
        "stremio:///player/eAEBJgDZ%2F3sidXJsIjoiaHR0cDovL2RvbWFpbi5yb290L3NvbWUvcGF0aCJ9AYANjw%3D%3D",
    );
    assert_eq!(
        sdl.external_player.playlist,
        Some(BASE64_HTTP_URL.to_owned()),
    );
    assert_eq!(sdl.external_player.streaming, Some(HTTP_STR_URL.to_owned()));
    assert_eq!(
        sdl.external_player.file_name,
        Some("playlist.m3u".to_string())
    );
}

#[test]
fn stream_deep_links_http_with_request_headers() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str(HTTP_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints {
            not_web_ready: false,
            binge_group: None,
            country_whitelist: None,
            filename: None,
            video_hash: None,
            video_size: None,
            proxy_headers: Some(StreamProxyHeaders {
                request: HashMap::from([("Authorization".to_string(), "my+token".to_string())]),
                response: Default::default(),
            }),
            other: Default::default(),
        },
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(sdl.player, "stremio:///player/eAEBcACP%2F3sidXJsIjoiaHR0cDovL2RvbWFpbi5yb290L3NvbWUvcGF0aCIsImJlaGF2aW9ySGludHMiOnsicHJveHlIZWFkZXJzIjp7InJlcXVlc3QiOnsiQXV0aG9yaXphdGlvbiI6Im15K3Rva2VuIn19fX3Y5Cjf".to_string());
    assert_eq!(
        sdl.external_player.streaming,
        Some(format!(
            "{}/proxy/{}",
            STREAMING_SERVER_URL,
            "d=http%3A%2F%2Fdomain.root&h=Authorization%3Amy%2Btoken/some/path",
        ))
    );
}

#[test]
fn stream_deep_links_http_with_request_response_headers_and_query_params() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str(HTTP_WITH_QUERY_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints {
            not_web_ready: false,
            binge_group: None,
            country_whitelist: None,
            filename: None,
            video_hash: None,
            video_size: None,
            proxy_headers: Some(StreamProxyHeaders {
                request: HashMap::from([("Authorization".to_string(), "my+token".to_string())]),
                response: HashMap::from([(
                    "Content-Type".to_string(),
                    "application/xml".to_string(),
                )]),
            }),
            other: Default::default(),
        },
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(sdl.player, "stremio:///player/eAEBsQBO%2F3sidXJsIjoiaHR0cDovL2RvbWFpbi5yb290L3NvbWUvcGF0aD9wYXJhbT1zb21lJmZvbz1iYXIiLCJiZWhhdmlvckhpbnRzIjp7InByb3h5SGVhZGVycyI6eyJyZXF1ZXN0Ijp7IkF1dGhvcml6YXRpb24iOiJteSt0b2tlbiJ9LCJyZXNwb25zZSI6eyJDb250ZW50LVR5cGUiOiJhcHBsaWNhdGlvbi94bWwifX19fT2nQI0%3D".to_string());
    assert_eq!(
        sdl.external_player.streaming,
        Some(format!(
            "{}/proxy/{}",
            STREAMING_SERVER_URL,
            "d=http%3A%2F%2Fdomain.root&h=Authorization%3Amy%2Btoken&r=Content-Type%3Aapplication%2Fxml/some/path?param=some&foo=bar",
        ))
    );
}

#[test]
fn stream_deep_links_torrent() {
    let info_hash = [
        0xdd, 0x82, 0x55, 0xec, 0xdc, 0x7c, 0xa5, 0x5f, 0xb0, 0xbb, 0xf8, 0x13, 0x23, 0xd8, 0x70,
        0x62, 0xdb, 0x1f, 0x6d, 0x1c,
    ];
    let file_idx = 0;
    let announce = vec!["http://bt1.archive.org:6969/announce".to_string()];
    let stream = Stream {
        source: StreamSource::Torrent {
            info_hash,
            file_idx: Some(file_idx),
            announce,
            file_must_include: vec![],
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(sdl.player, "stremio:///player/eAEBdwCI%2F3siaW5mb0hhc2giOiJkZDgyNTVlY2RjN2NhNTVmYjBiYmY4MTMyM2Q4NzA2MmRiMWY2ZDFjIiwiZmlsZUlkeCI6MCwiYW5ub3VuY2UiOlsiaHR0cDovL2J0MS5hcmNoaXZlLm9yZzo2OTY5L2Fubm91bmNlIl19ndAlsw%3D%3D".to_string());
    assert_eq!(
        sdl.external_player.playlist,
        Some(format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            BASE64.encode(format!(
                "#EXTM3U\n#EXTINF:0\n{}",
                format_args!(
                    "{}/{}/{}?tr={}",
                    STREAMING_SERVER_URL,
                    hex::encode(info_hash),
                    file_idx,
                    utf8_percent_encode(
                        "http://bt1.archive.org:6969/announce",
                        URI_COMPONENT_ENCODE_SET,
                    ),
                )
            ))
        ))
    );
    assert_eq!(
        sdl.external_player.streaming,
        Some(format!(
            "{}/{}/{}?tr={}",
            STREAMING_SERVER_URL,
            hex::encode(info_hash),
            file_idx,
            utf8_percent_encode(
                "http://bt1.archive.org:6969/announce",
                URI_COMPONENT_ENCODE_SET,
            ),
        ))
    );
    assert_eq!(
        sdl.external_player.file_name,
        Some("playlist.m3u".to_string())
    );
}

#[test]
fn stream_deep_links_torrent_without_file_index() {
    let info_hash = [
        0xdd, 0x82, 0x55, 0xec, 0xdc, 0x7c, 0xa5, 0x5f, 0xb0, 0xbb, 0xf8, 0x13, 0x23, 0xd8, 0x70,
        0x62, 0xdb, 0x1f, 0x6d, 0x1c,
    ];
    let announce = vec!["http://bt1.archive.org:6969/announce".to_string()];
    let stream = Stream {
        source: StreamSource::Torrent {
            info_hash,
            file_idx: None,
            announce,
            file_must_include: vec![],
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(sdl.player, "stremio:///player/eAEBegCF%2F3siaW5mb0hhc2giOiJkZDgyNTVlY2RjN2NhNTVmYjBiYmY4MTMyM2Q4NzA2MmRiMWY2ZDFjIiwiZmlsZUlkeCI6bnVsbCwiYW5ub3VuY2UiOlsiaHR0cDovL2J0MS5hcmNoaXZlLm9yZzo2OTY5L2Fubm91bmNlIl19LmMnPg%3D%3D".to_string());
    assert_eq!(
        sdl.external_player.playlist,
        Some(format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            BASE64.encode(format!(
                "#EXTM3U\n#EXTINF:0\n{}",
                format_args!(
                    "{}/{}/{}?tr={}",
                    STREAMING_SERVER_URL,
                    hex::encode(info_hash),
                    -1,
                    utf8_percent_encode(
                        "http://bt1.archive.org:6969/announce",
                        URI_COMPONENT_ENCODE_SET,
                    ),
                )
            ))
        ))
    );
    assert_eq!(
        sdl.external_player.streaming,
        Some(format!(
            "{}/{}/{}?tr={}",
            STREAMING_SERVER_URL,
            hex::encode(info_hash),
            -1,
            utf8_percent_encode(
                "http://bt1.archive.org:6969/announce",
                URI_COMPONENT_ENCODE_SET,
            ),
        ))
    );
    assert_eq!(
        sdl.external_player.file_name,
        Some("playlist.m3u".to_string())
    );
}

#[test]
fn stream_deep_links_external() {
    let stream = Stream {
        source: StreamSource::External {
            external_url: Some(Url::from_str(HTTP_STR_URL).unwrap()),
            android_tv_url: None,
            tizen_url: None,
            webos_url: None,
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(&sdl.player, "stremio:///player/eAEBLgDR%2F3siZXh0ZXJuYWxVcmwiOiJodHRwOi8vZG9tYWluLnJvb3Qvc29tZS9wYXRoIn2LPRDS");
    assert_eq!(
        sdl.external_player.web,
        Some(Url::from_str(HTTP_STR_URL).unwrap()),
    );
    assert_eq!(sdl.external_player.file_name, None);
}

#[test]
fn stream_deep_links_youtube() {
    let stream = Stream {
        source: StreamSource::YouTube {
            yt_id: YT_ID.to_string(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(
        sdl.player,
        "stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5".to_string()
    );
    assert_eq!(
        sdl.external_player.playlist,
        Some(format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            BASE64.encode(format!(
                "#EXTM3U\n#EXTINF:0\n{}/yt/{}",
                STREAMING_SERVER_URL, YT_ID
            ))
        ))
    );
    assert_eq!(
        sdl.external_player.file_name,
        Some("playlist.m3u".to_string())
    );
}

#[test]
fn stream_deep_links_player_frame() {
    let stream = Stream {
        source: StreamSource::PlayerFrame {
            player_frame_url: Url::from_str(HTTP_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((&stream, &streaming_server_url, &settings));
    assert_eq!(&sdl.player, "stremio:///player/eAEBMQDO%2F3sicGxheWVyRnJhbWVVcmwiOiJodHRwOi8vZG9tYWluLnJvb3Qvc29tZS9wYXRoIn2%2F2hHn");
    assert_eq!(sdl.external_player.playlist, None);
    assert_eq!(sdl.external_player.file_name, None);
}

#[test]
fn stream_deep_links_requests() {
    let stream = Stream {
        source: StreamSource::YouTube {
            yt_id: YT_ID.to_string(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let stream_request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("stream", "movie", format!("yt_id:{YT_ID}").as_str()),
    };
    let meta_request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", "movie", format!("yt_id:{YT_ID}").as_str()),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let sdl = StreamDeepLinks::from((
        &stream,
        &stream_request,
        &meta_request,
        &streaming_server_url,
        &settings,
    ));
    assert_eq!(sdl.player, format!(
        "stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5/http%3A%2F%2Fdomain.root%2F/http%3A%2F%2Fdomain.root%2F/movie/yt_id%3A{}/yt_id%3A{}",
        YT_ID, YT_ID
    ));
    assert_eq!(
        sdl.external_player.playlist,
        Some(format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            BASE64.encode(format!(
                "#EXTM3U\n#EXTINF:0\n{}/yt/{}",
                STREAMING_SERVER_URL, YT_ID
            ))
        ))
    );
    assert_eq!(
        sdl.external_player.file_name,
        Some("playlist.m3u".to_string())
    );
}
