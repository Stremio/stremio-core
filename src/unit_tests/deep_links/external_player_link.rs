use crate::constants::{BASE64, URI_COMPONENT_ENCODE_SET};
use crate::deep_links::ExternalPlayerLink;
use crate::types::profile::Settings;
use crate::types::resource::{Stream, StreamSource};
use base64::Engine;
use percent_encoding::utf8_percent_encode;
use std::convert::TryFrom;
use std::str::FromStr;
use url::Url;

const MAGNET_STR_URL: &str = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&tr=http%3A%2F%2Fbt1.archive.org%3A6969%2Fannounce";
const HTTP_STR_URL: &str = "http://domain.root/path";
const BASE64_HTTP_URL: &str = "data:application/octet-stream;charset=utf-8;base64,I0VYVE0zVQojRVhUSU5GOjAKaHR0cDovL2RvbWFpbi5yb290L3BhdGg=";
const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11471";

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
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(epl.href, Some(MAGNET_STR_URL.to_owned()));
    assert_eq!(epl.file_name, None);
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
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(epl.href, Some(BASE64_HTTP_URL.to_owned()));
    assert_eq!(epl.file_name, Some("playlist.m3u".to_string()));
}

#[test]
fn external_player_link_torrent() {
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
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(
        epl.href,
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
                        URI_COMPONENT_ENCODE_SET
                    )
                )
            ))
        ))
    );
    assert_eq!(epl.download, Some(MAGNET_STR_URL.to_owned()));
    assert_eq!(epl.file_name, Some("playlist.m3u".to_string()));
}

#[test]
fn external_player_link_external() {
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
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(epl.href, Some(HTTP_STR_URL.to_owned()));
    assert_eq!(epl.file_name, None);
}

#[test]
fn external_player_link_youtube() {
    let yt_id = "aqz-KE-bpKQ";
    let stream = Stream {
        source: StreamSource::YouTube {
            yt_id: yt_id.to_string(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(
        epl.href,
        Some(format!(
            "data:application/octet-stream;charset=utf-8;base64,{}",
            BASE64.encode(format!(
                "#EXTM3U\n#EXTINF:0\n{}/yt/{}",
                STREAMING_SERVER_URL, yt_id
            ))
        ))
    );
    assert_eq!(epl.file_name, Some("playlist.m3u".to_string()));
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
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let settings = Settings::default();
    let epl = ExternalPlayerLink::try_from((&stream, &streaming_server_url, &settings)).unwrap();
    assert_eq!(epl.href, Some(HTTP_STR_URL.to_owned()));
    assert_eq!(epl.file_name, None);
}
