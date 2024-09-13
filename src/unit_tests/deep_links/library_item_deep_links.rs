use chrono::Utc;
use once_cell::sync::Lazy;
use stremio_serde_hex::{SerHex, Strict};

use crate::constants::STREAMING_SERVER_URL;
use crate::deep_links::LibraryItemDeepLinks;
use crate::deep_links::OpenPlayerLink;
use crate::types::library::LibraryItem;
use crate::types::library::LibraryItemState;
use crate::types::profile::Settings;
use crate::types::resource::Stream;
use crate::types::resource::StreamBehaviorHints;
use crate::types::resource::StreamSource;
use crate::types::resource::{MetaItemBehaviorHints, PosterShape};
use crate::types::streams::StreamsItem;

const META_DETAILS_VIDEOS: &str = "stremio:///detail/series/tt13622776";

static INFUSE_PLAYER_SETTINGS: Lazy<Settings> = Lazy::new(|| Settings {
    player_type: Some("infuse".to_owned()),
    ..Settings::default()
});

static TORRENT_STREAMS_ITEM: Lazy<StreamsItem> = Lazy::new(|| {
    let stream = Stream {
        source: StreamSource::Torrent {
            info_hash: SerHex::<Strict>::from_hex("df2c94aec35f97943c4e432f25081b590cd35326")
                .unwrap(),
            file_idx: Some(0),
            announce: vec![],
            file_must_include: vec![],
        },
        name: Some("PaidTV".to_owned()),
        description: Some("Ahsoka S01E05 Part Five 1080p".to_owned()),
        thumbnail: None,
        subtitles: vec![],
        behavior_hints: StreamBehaviorHints::default(),
    };

    StreamsItem {
        stream,
        r#type: "series".to_owned(),
        meta_id: "tt13622776".to_owned(),
        video_id: "tt13622776:1:5".to_owned(),
        meta_transport_url: "https://v3-cinemeta.strem.io/manifest.json"
            .parse()
            .unwrap(),
        stream_transport_url: "https://torrentio.strem.fun/qualityfilter=1080p,720p/manifest.json"
            .parse()
            .unwrap(),
        state: None,
        mtime: Utc::now(),
    }
});

#[test]
fn library_item_deep_links_no_video() {
    let lib_item = LibraryItem {
        id: "tt13622776".to_string(),
        name: "Ahsoka".to_string(),
        r#type: "series".to_string(),
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
            watched: None,
            no_notif: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        None,
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );
    assert_eq!(lidl.meta_details_streams, None);
    assert_eq!(
        lidl.player, None,
        "We do not have a video so we cannot have a Stream pulled from the StreamBucket"
    );
    assert_eq!(
        lidl.external_player, None,
        "We do not have a video so we cannot have a Stream pulled from the StreamBucket"
    );
}

#[test]
fn library_item_deep_links_state_video_id_no_time_offset_infuse_player() {
    // With state.video_id
    let lib_item = LibraryItem {
        id: "tt13622776".to_string(),
        name: "Ahsoka".to_string(),
        r#type: "series".to_string(),
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
            video_id: Some("tt13622776:1:5".to_string()),
            watched: None,
            no_notif: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        // We have a video so we can have a Stream pulled from the StreamBucket!
        Some(&*TORRENT_STREAMS_ITEM),
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );
    assert_eq!(lidl.meta_details_streams, None);
    // with a stream and StreamSource::Torrent we should get an external_player link
    assert!(matches!(
        lidl.external_player
            .expect("Should have external player")
            .open_player,
            Some(OpenPlayerLink {
                ios: Some(ios),
                ..
            }) if ios.starts_with("infuse://")
    ));
    // with a Stream we should get a player link
    assert!(matches!(lidl.player, Some(player) if player.starts_with("stremio:///player")));
}

#[test]
fn library_item_deep_links_state_video_id() {
    // With state.video_id
    let lib_item = LibraryItem {
        id: "tt13622776".to_string(),
        name: "Ahsoka".to_string(),
        r#type: "series".to_string(),
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
            video_id: Some("tt13622776:1:5".to_string()),
            watched: None,
            no_notif: true,
        },
        behavior_hints: Default::default(),
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        // We have a video so we can have a Stream pulled from the StreamBucket!
        Some(&*TORRENT_STREAMS_ITEM),
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(
        lidl.meta_details_videos,
        Some(META_DETAILS_VIDEOS.to_string())
    );

    assert!(matches!(lidl.player, Some(player) if player.starts_with("stremio:///player")));
    // with a stream and StreamSource::Torrent we should get an external_player link
    assert!(matches!(
        lidl.external_player
            .expect("Should have external player")
            .open_player,
            Some(OpenPlayerLink {
                ios: Some(ios),
                ..
            }) if ios.starts_with("infuse://")
    ));
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
            watched: None,
            no_notif: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("tt13622776:1:5".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        // We have a video so we can have a Stream pulled from the StreamBucket!
        Some(&*TORRENT_STREAMS_ITEM),
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/tt13622776%3A1%3A5".to_string())
    );
    assert!(matches!(lidl.player, Some(player) if player.starts_with("stremio:///player")));
    // with a stream and StreamSource::Torrent we should get an external_player link
    assert!(matches!(
        lidl.external_player
            .expect("Should have external player")
            .open_player,
            Some(OpenPlayerLink {
                ios: Some(ios),
                ..
            }) if ios.starts_with("infuse://")
    ));
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
            watched: None,
            no_notif: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        // We have a video so we can have a Stream pulled from the StreamBucket!
        Some(&*TORRENT_STREAMS_ITEM),
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/video_id".to_string())
    );
    assert!(matches!(lidl.player, Some(player) if player.starts_with("stremio:///player")));
    // with a stream and StreamSource::Torrent we should get an external_player link
    assert!(matches!(
        lidl.external_player
            .expect("Should have external player")
            .open_player,
            Some(OpenPlayerLink {
                ios: Some(ios),
                ..
            }) if ios.starts_with("infuse://")
    ));
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
            watched: None,
            no_notif: true,
        },
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: Some("bh_video_id".to_string()),
            featured_video_id: None,
            has_scheduled_videos: false,
            other: Default::default(),
        },
    };
    let lidl = LibraryItemDeepLinks::from((
        &lib_item,
        // We have a video so we can have a Stream pulled from the StreamBucket!
        Some(&*TORRENT_STREAMS_ITEM),
        Some(&*STREAMING_SERVER_URL),
        &*INFUSE_PLAYER_SETTINGS,
    ));
    assert_eq!(lidl.meta_details_videos, None);
    assert_eq!(
        lidl.meta_details_streams,
        Some("stremio:///detail/movie/tt1254207/bh_video_id".to_string())
    );
    assert!(matches!(lidl.player, Some(player) if player.starts_with("stremio:///player")));
    // with a stream and StreamSource::Torrent we should get an external_player link
    assert!(matches!(
        lidl.external_player
            .expect("Should have external player")
            .open_player,
            Some(OpenPlayerLink {
                ios: Some(ios),
                ..
            }) if ios.starts_with("infuse://")
    ));
}
