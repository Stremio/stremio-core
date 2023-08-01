use crate::deep_links::LibraryItemDeepLinks;
use crate::types::library::LibraryItem;
use crate::types::library::LibraryItemState;
use crate::types::resource::{MetaItemBehaviorHints, PosterShape};
use chrono::Utc;
use std::convert::TryFrom;

const META_DETAILS_VIDEOS: &str = "stremio:///detail/movie/tt1254207";

#[test]
fn library_item_deep_links() {
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "movie".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 0,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: None,
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );
    assert_eq!(lidl.meta_details_streams, None);
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}

#[test]
fn library_item_deep_links_state_video_id_no_time_offset() {
    // With state.video_id
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "movie".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 0,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: Some("video_id".to_string()),
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );
    assert_eq!(lidl.meta_details_streams, None);
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}

#[test]
fn library_item_deep_links_state_video_id() {
    // With state.video_id
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "movie".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 1,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: Some("video_id".to_string()),
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/video_id".to_string())
    );
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}

#[test]
fn library_item_deep_links_behavior_hints_default_video_id() {
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "movie".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 0,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: None,
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/bh_video_id".to_string())
    );
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}

#[test]
fn library_item_deep_links_state_and_behavior_hints_default_video_id() {
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "Big Buck Bunny".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 1,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: Some("video_id".to_string()),
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/video_id".to_string())
    );
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}

#[test]
fn library_item_deep_links_state_no_time_offset_and_behavior_hints_default_video_id() {
    let lib_item = LibraryItem {
        id: "tt1254207".to_string(),
        name: "Big Buck Bunny".to_string(),
        r#type: "movie".to_string(),
        poster: None,
        poster_shape: PosterShape::Poster,
        removed: false,
        temp: true,
        ctime: None,
        mtime: Utc::now(),
        state: LibraryItemState {
            last_watched: None,
            time_watched: 999,
            time_offset: 0,
            overall_time_watched: 999,
            times_watched: 999,
            flagged_watched: 1,
            duration: 0,
            video_id: Some("video_id".to_string()),
            stream: None,
            watched: None,
            last_video_released: None,
            notifications_disabled: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::try_from(&lib_item).unwrap();
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/bh_video_id".to_string())
    );
    assert_eq!(lidl.player, None);
    assert_eq!(lidl.external_player, None);
}
