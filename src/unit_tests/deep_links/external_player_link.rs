use crate::constants::{BASE64, URI_COMPONENT_ENCODE_SET};
use crate::deep_links::ExternalPlayerLink;
use crate::types::profile::Settings;
use crate::types::resource::{Stream, StreamSource};
use base64::Engine;
use percent_encoding::utf8_percent_encode;
use std::str::FromStr;
use url::Url;

const MAGNET_STR_URL: &str = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&tr=http%3A%2F%2Fbt1.archive.org%3A6969%2Fannounce";
const HTTP_STR_URL: &str = "http://domain.root/path";
const BASE64_HTTP_URL: &str = "data:application/octet-stream;charset=utf-8;base64,I0VYVE0zVQojRVhUSU5GOjAKaHR0cDovL2RvbWFpbi5yb290L3BhdGg=";
const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11470";

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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(epl.download, Some(MAGNET_STR_URL.to_owned()));
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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(epl.playlist, Some(BASE64_HTTP_URL.to_owned()));
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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(
        epl.playlist,
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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(epl.web, Some(Url::from_str(HTTP_STR_URL).unwrap()));
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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(
        epl.playlist,
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
    let epl = ExternalPlayerLink::from((&stream, &streaming_server_url, &settings));
    assert_eq!(epl.playlist, None);
    assert_eq!(epl.file_name, None);
}

#[test]
fn external_player_link_with_vlc_player() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str("http://example.com/stream").unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());

    let settings = Settings {
        player_type: Some("vlc".to_string()),
        streaming_server_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
        ..Default::default()
    };

    let epl = ExternalPlayerLink::from((&stream, streaming_server_url.as_ref(), &settings));

    let open_player = epl.open_player.as_ref().unwrap();

    assert_eq!(
        open_player.android,
        Some("intent://example.com/stream#Intent;package=org.videolan.vlc;type=video;scheme=https;end".to_string())
    );
    assert_eq!(
        open_player.ios,
        Some("vlc-x-callback://x-callback-url/stream?url=http://example.com/stream".to_string())
    );
}

#[test]
fn external_player_link_with_mxplayer() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str("http://example.com/stream").unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());

    let settings = Settings {
        player_type: Some("mxplayer".to_string()),
        streaming_server_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
        ..Default::default()
    };

    let epl = ExternalPlayerLink::from((&stream, streaming_server_url.as_ref(), &settings));

    assert_eq!(
        epl.open_player.unwrap().android,
        Some("intent://example.com/stream#Intent;package=com.mxtech.videoplayer.ad;type=video;scheme=https;end".to_string())
    );
}

#[test]
fn external_player_link_with_justplayer() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str("http://example.com/stream").unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());

    let settings = Settings {
        player_type: Some("justplayer".to_string()),
        streaming_server_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
        ..Default::default()
    };

    let epl = ExternalPlayerLink::from((&stream, streaming_server_url.as_ref(), &settings));

    assert_eq!(
        epl.open_player.unwrap().android,
        Some("intent://example.com/stream#Intent;package=com.brouken.player;type=video;scheme=https;end".to_string())
    );
}

#[test]
fn external_player_link_with_outplayer() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str("http://example.com/stream").unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());

    let settings = Settings {
        player_type: Some("outplayer".to_string()),
        streaming_server_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
        ..Default::default()
    };

    let epl = ExternalPlayerLink::from((&stream, streaming_server_url.as_ref(), &settings));

    assert_eq!(
        epl.open_player.unwrap().ios,
        Some("outplayer://example.com/stream".to_string())
    );
}

#[test]
fn external_player_link_with_infuse() {
    let stream = Stream {
        source: StreamSource::Url {
            url: Url::from_str("http://example.com/stream").unwrap(),
        },
        name: None,
        description: None,
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: Default::default(),
    };

    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());

    let settings = Settings {
        player_type: Some("infuse".to_string()),
        streaming_server_url: Url::parse(STREAMING_SERVER_URL).unwrap(),
        ..Default::default()
    };

    let epl = ExternalPlayerLink::from((&stream, streaming_server_url.as_ref(), &settings));

    let open_player = epl.open_player.as_ref().unwrap();

    assert_eq!(
        open_player.ios,
        Some("infuse://x-callback-url/play?url=http://example.com/stream".to_string())
    );
}
