use crate::deep_links::MetaItemDeepLinks;
use crate::types::addon::{ResourcePath, ResourceRequest};
use crate::types::resource::{MetaItem, MetaItemBehaviorHints, MetaItemPreview, PosterShape};
use std::str::FromStr;
use url::Url;

#[test]
fn meta_item_deep_links() {
    let preview = MetaItemPreview {
        id: "tt1254207".to_string(),
        r#type: "movie".to_string(),
        name: "Big Buck Bunny".to_string(),
        poster: None,
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: PosterShape::Poster,
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints::default(),
    };
    let item = MetaItem {
        preview: preview.clone(),
        videos: vec![],
    };
    let request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", preview.r#type.as_ref(), preview.id.as_ref()),
    };

    let preview_midl = MetaItemDeepLinks::from((&preview, &request));
    let midl = MetaItemDeepLinks::from((&item, &request));
    assert_eq!(preview_midl, midl);
    assert_eq!(
        midl.meta_details_videos,
        Some("stremio:///detail/movie/tt1254207".to_string())
    );
    assert_eq!(midl.meta_details_streams, None);
    assert_eq!(midl.player, None);
}

#[test]
fn meta_item_deep_links_behavior_hints() {
    let preview = MetaItemPreview {
        id: "tt1254207".to_string(),
        r#type: "movie".to_string(),
        name: "Big Buck Bunny".to_string(),
        poster: None,
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: PosterShape::Poster,
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let item = MetaItem {
        preview: preview.clone(),
        videos: vec![],
    };
    let request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", preview.r#type.as_ref(), preview.id.as_ref()),
    };

    let preview_midl = MetaItemDeepLinks::from((&preview, &request));
    let midl = MetaItemDeepLinks::from((&item, &request));
    assert_eq!(preview_midl, midl);
    assert_eq!(midl.meta_details_videos, None);
    assert_eq!(
        midl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/bh_video_id".to_string())
    );
    assert_eq!(midl.player, None);
}

#[test]
fn meta_item_deep_links_behavior_hints_yt_id() {
    let preview = MetaItemPreview {
        id: "tt1254207".to_string(),
        r#type: "movie".to_string(),
        name: "Big Buck Bunny".to_string(),
        poster: None,
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: PosterShape::Poster,
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("yt_id:UCSMOQeBJ2RAnuFungnQOxLg:aqz-KE-bpKQ".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let item = MetaItem {
        preview: preview.clone(),
        videos: vec![],
    };
    let request = ResourceRequest {
        base: Url::from_str("http://domain.root").unwrap(),
        path: ResourcePath::without_extra("meta", preview.r#type.as_ref(), preview.id.as_ref()),
    };

    let preview_midl = MetaItemDeepLinks::from((&preview, &request));
    let midl = MetaItemDeepLinks::from((&item, &request));
    assert_eq!(preview_midl, midl);
    assert_eq!(midl.meta_details_videos, None);
    assert_eq!(
        midl.meta_details_streams,
        Some(
            "stremio:///detail/movie/tt1254207/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3Aaqz-KE-bpKQ"
                .to_string()
        )
    );
    assert_eq!(midl.player, Some("stremio:///player/eAEBFgDp%2F3sieXRJZCI6ImFxei1LRS1icEtRIn1RRQb5/http%3A%2F%2Fdomain.root%2F/http%3A%2F%2Fdomain.root%2F/movie/tt1254207/yt_id%3AUCSMOQeBJ2RAnuFungnQOxLg%3Aaqz-KE-bpKQ".to_string()));
}
