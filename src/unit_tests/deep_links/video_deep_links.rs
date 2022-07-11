use crate::deep_links::{ExternalPlayerLink, VideoDeepLinks};
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::resource::Video;
use std::convert::TryFrom;
use std::str::FromStr;
use url::Url;

#[test]
fn video_deep_links() {
    let video = Video {
        id: "yt_id:UCSMOQeBJ2RAnuFungnQOxLg:aqz-KE-bpKQ".to_string(),
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
        path: ResourcePath::without_extra("meta", "movie", "yt_id:aqz-KE-bpKQ"),
    };
    let vdl = VideoDeepLinks::try_from((&video, &request)).unwrap();
    assert_eq!(vdl.meta_details_streams, "stremio:///detail/movie/yt_id%3Aaqz-KE-bpKQ/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3Aaqz-KE-bpKQ".to_string());
    assert_eq!(vdl.player, Some("stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5/http%3A%2F%2Fdomain.root%2F/http%3A%2F%2Fdomain.root%2F/movie/yt_id%3Aaqz-KE-bpKQ/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3Aaqz-KE-bpKQ".to_string()));
    assert_eq!(
        vdl.external_player,
        Some(ExternalPlayerLink {
            href: "https://www.youtube.com/watch?v=aqz-KE-bpKQ".to_string(),
            download: None
        })
    );
}
