use crate::deep_links::ExternalPlayerLink;
use crate::types::resource::{Stream, StreamSource};
use std::convert::TryFrom;
use std::str::FromStr;
use url::Url;

const MAGNET_STR_URL: &str = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c";
const HTTP_STR_URL: &str = "http://domain.root/path";
const BASE64_HTTP_URL: &str = "data:application/octet-stream;charset=utf-8;base64,I0VYVE0zVQojRVhUSU5GOjAKaHR0cDovL2RvbWFpbi5yb290L3BhdGg=";
#[test]
fn external_player_link_magnet() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(epl.href, MAGNET_STR_URL);
    assert_eq!(epl.download, None);
}

#[test]
fn external_player_link_http() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(epl.href, BASE64_HTTP_URL);
    assert_eq!(epl.download, Some("playlist.m3u".to_string()));
}

#[test]
fn external_player_link_torrent() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(epl.href, MAGNET_STR_URL);
    assert_eq!(epl.download, None);
}

#[test]
fn external_player_link_external() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(epl.href, HTTP_STR_URL);
    assert_eq!(epl.download, None);
}

#[test]
fn external_player_link_youtube() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(
        epl.href,
        "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string()
    );
    assert_eq!(epl.download, None);
}

#[test]
fn external_player_link_player_frame() {
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
    let epl = ExternalPlayerLink::try_from(&stream).unwrap();
    assert_eq!(epl.href, HTTP_STR_URL);
    assert_eq!(epl.download, None);
}
