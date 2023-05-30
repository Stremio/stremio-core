use crate::constants::BASE64;
use crate::deep_links::{ExternalPlayerLink, OpenPlayerLink, VideoDeepLinks};
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::resource::Video;
use base64::Engine;
use regex::Regex;
use std::convert::TryFrom;
use std::str::FromStr;
use url::Url;

const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11471/";
const YT_ID: &str = "aqz-KE-bpKQ";

#[test]
fn video_deep_links() {
    let http_regex = Regex::new(r"https?://").unwrap();
    let video = Video {
        id: format!("yt_id:UCSMOQeBJ2RAnuFungnQOxLg:{YT_ID}"),
        title: "Big Buck Bunny".to_string(),
        released: None,
        overview: None,
        thumbnail: None,
        streams: vec![],
        series_info: None,
        trailer_streams: vec![],
    };
    let request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", "movie", format!("yt_id:{YT_ID}").as_str()),
    };
    let streaming_server_url = Some(Url::parse(STREAMING_SERVER_URL).unwrap());
    let streaming_server_yt = format!("{}yt/{}", STREAMING_SERVER_URL, YT_ID,);
    let vdl = VideoDeepLinks::try_from((&video, &request, &streaming_server_url)).unwrap();
    assert_eq!(
        vdl.meta_details_streams,
        format!(
            "stremio:///detail/movie/yt_id%3A{}/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3A{}",
            YT_ID, YT_ID
        )
    );
    assert_eq!(vdl.player, Some(format!(
        "stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5/http%3A%2F%2Fdomain.root%2F/http%3A%2F%2Fdomain.root%2F/movie/yt_id%3A{}/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3A{}",
        YT_ID, YT_ID
    )));
    assert_eq!(
        vdl.external_player,
        Some(ExternalPlayerLink {
            href: Some(format!(
                "data:application/octet-stream;charset=utf-8;base64,{}",
                BASE64.encode(format!(
                    "#EXTM3U\n#EXTINF:0\n{}",
                    streaming_server_yt
                ))
            )),
            download: Some(format!("https://youtube.com/watch?v={}", YT_ID)),
            streaming: Some(format!("{}yt/{}", STREAMING_SERVER_URL, YT_ID)),
            file_name: Some("playlist.m3u".to_string()),
            choose: Some(OpenPlayerLink {
                android: Some(format!(
                    "intent:{}#Intent;type=video/any;scheme=https;end",
                    streaming_server_yt,
                )),
                ios: None,
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            vlc: Some(OpenPlayerLink {
                ios: Some(format!(
                    "vlc-x-callback://x-callback-url/stream?url={}",
                    streaming_server_yt,
                )),
                android: Some(format!(
                    "intent:{}#Intent;package=org.videolan.vlc;type=video;scheme=https;end",
                    streaming_server_yt,
                )),
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            mxplayer: Some(OpenPlayerLink {
                android: Some(format!(
                    "intent:{}#Intent;package=com.mxtech.videoplayer.ad;type=video;scheme=https;end",
                    streaming_server_yt,
                )),
                ios: None,
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            justplayer: Some(OpenPlayerLink {
                android: Some(format!(
                    "intent:{}#Intent;package=com.brouken.player;type=video;scheme=https;end",
                    streaming_server_yt,
                )),
                ios: None,
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            outplayer: Some(OpenPlayerLink {
                ios: Some(format!("{}", http_regex.replace(&streaming_server_yt, "outplayer://"))),
                android: None,
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            infuse: Some(OpenPlayerLink {
                ios: Some(format!("{}", http_regex.replace(&streaming_server_yt, "infuse://"))),
                android: None,
                windows: None,
                macos: None,
                linux: None,
                tizen: None,
                webos: None,
                chromeos: None,
                roku: None,
            }),
            ..Default::default()
        })
    );
}
