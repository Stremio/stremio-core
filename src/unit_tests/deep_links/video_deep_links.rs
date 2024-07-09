use crate::constants::BASE64;
use crate::deep_links::{ExternalPlayerLink, VideoDeepLinks};
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::profile::Settings;
use crate::types::resource::Video;
use base64::Engine;
use std::str::FromStr;
use url::Url;

const STREAMING_SERVER_URL: &str = "http://127.0.0.1:11470/";
const YT_ID: &str = "aqz-KE-bpKQ";

#[test]
fn video_deep_links() {
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
    let settings = Settings::default();
    let vdl = VideoDeepLinks::from((&video, &request, &streaming_server_url, &settings));
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
            playlist: Some(format!(
                "data:application/octet-stream;charset=utf-8;base64,{}",
                BASE64.encode(format!(
                    "#EXTM3U\n#EXTINF:0\n{}yt/{}",
                    STREAMING_SERVER_URL, YT_ID
                ))
            )),
            download: Some(format!("https://youtube.com/watch?v={}", YT_ID)),
            streaming: Some(format!("{}yt/{}", STREAMING_SERVER_URL, YT_ID)),
            file_name: Some("playlist.m3u".to_string()),
            ..Default::default()
        })
    );
}
