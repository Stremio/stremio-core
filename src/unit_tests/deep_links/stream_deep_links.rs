use crate::deep_links::StreamDeepLinks;
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::resource::{Stream, StreamSource};
use std::convert::TryFrom;
use std::str::FromStr;
use url::Url;

const MAGNET_STR_URL: &str = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c";
const HTTP_STR_URL: &str = "http://domain.root/path";
const BASE64_HTTP_URL: &str = "data:application/octet-stream;charset=utf-8;base64,I0VYVE0zVQojRVhUSU5GOjAKaHR0cDovL2RvbWFpbi5yb290L3BhdGg=";

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
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(sdl.player, "stremio:///player/eAEBRgC5%2F3sidXJsIjoibWFnbmV0Oj94dD11cm46YnRpaDpkZDgyNTVlY2RjN2NhNTVmYjBiYmY4MTMyM2Q4NzA2MmRiMWY2ZDFjIn0%2BMhZF".to_string());
    assert_eq!(sdl.external_player.href, MAGNET_STR_URL);
    assert_eq!(sdl.external_player.download, None);
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
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(
        sdl.player,
        "stremio:///player/eAEBIQDe%2F3sidXJsIjoiaHR0cDovL2RvbWFpbi5yb290L3BhdGgifcEEC6w%3D"
            .to_string()
    );
    assert_eq!(sdl.external_player.href, BASE64_HTTP_URL);
    assert_eq!(
        sdl.external_player.download,
        Some("playlist.m3u".to_string())
    );
}

#[test]
fn stream_deep_links_torrent() {
    let stream = Stream {
        source: StreamSource::Torrent {
            info_hash: [
                0xdd, 0x82, 0x55, 0xec, 0xdc, 0x7c, 0xa5, 0x5f, 0xb0, 0xbb, 0xf8, 0x13, 0x23, 0xd8,
                0x70, 0x62, 0xdb, 0x1f, 0x6d, 0x1c,
            ],
            file_idx: None,
            announce: vec![],
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(sdl.player, "stremio:///player/eAEBVACr%2F3siaW5mb0hhc2giOiJkZDgyNTVlY2RjN2NhNTVmYjBiYmY4MTMyM2Q4NzA2MmRiMWY2ZDFjIiwiZmlsZUlkeCI6bnVsbCwiYW5ub3VuY2UiOltdfVAyGnc%3D".to_string());
    assert_eq!(sdl.external_player.href, MAGNET_STR_URL);
    assert_eq!(sdl.external_player.download, None);
}

#[test]
fn stream_deep_links_external() {
    let stream = Stream {
        source: StreamSource::External {
            external_url: Url::from_str(HTTP_STR_URL).unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(sdl.player, "stremio:///player/eAEBKQDW%2F3siZXh0ZXJuYWxVcmwiOiJodHRwOi8vZG9tYWluLnJvb3QvcGF0aCJ9OoEO7w%3D%3D".to_string());
    assert_eq!(sdl.external_player.href, HTTP_STR_URL);
    assert_eq!(sdl.external_player.download, None);
}

#[test]
fn stream_deep_links_youtube() {
    let stream = Stream {
        source: StreamSource::YouTube {
            yt_id: "aqz-KE-bpKQ".to_string(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(
        sdl.player,
        "stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5".to_string()
    );
    assert_eq!(
        sdl.external_player.href,
        "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string()
    );
    assert_eq!(sdl.external_player.download, None);
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
    let sdl = StreamDeepLinks::try_from(&stream).unwrap();
    assert_eq!(sdl.player, "stremio:///player/eAEBLADT%2F3sicGxheWVyRnJhbWVVcmwiOiJodHRwOi8vZG9tYWluLnJvb3QvcGF0aCJ9abUQBA%3D%3D".to_string());
    assert_eq!(sdl.external_player.href, HTTP_STR_URL);
    assert_eq!(sdl.external_player.download, None);
}

#[test]
fn stream_deep_links_requests() {
    let stream = Stream {
        source: StreamSource::YouTube {
            yt_id: "aqz-KE-bpKQ".to_string(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let stream_request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("stream", "movie", "yt_id:aqz-KE-bpKQ"),
    };
    let meta_request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", "movie", "yt_id:aqz-KE-bpKQ"),
    };

    let sdl = StreamDeepLinks::try_from((&stream, &stream_request, &meta_request)).unwrap();
    assert_eq!(sdl.player, "stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5/http%3A%2F%2Fdomain.root%2F/http%3A%2F%2Fdomain.root%2F/movie/yt_id%3Aaqz-KE-bpKQ/yt_id%3Aaqz-KE-bpKQ".to_string());
    assert_eq!(
        sdl.external_player.href,
        "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string()
    );
    assert_eq!(sdl.external_player.download, None);
}
