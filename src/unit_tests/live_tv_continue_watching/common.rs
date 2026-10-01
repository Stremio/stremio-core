use std::any::Any;

use chrono::{TimeZone, Utc};
use futures::future;
use url::Url;

use crate::{
    constants::META_RESOURCE_NAME,
    runtime::{EnvFutureExt, TryEnvFuture},
    types::{
        addon::{Descriptor, Manifest, ManifestBehaviorHints, ResourceResponse},
        library::{LibraryItem, LibraryItemState},
        resource::{MetaItem, MetaItemPreview, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request},
};

pub fn epg_info(start: (u32, u32), end: (u32, u32)) -> VideoEpgInfo {
    VideoEpgInfo {
        start_time: Utc
            .with_ymd_and_hms(2026, 7, 2, start.0, start.1, 0)
            .unwrap(),
        end_time: Utc.with_ymd_and_hms(2026, 7, 2, end.0, end.1, 0).unwrap(),
        runtime: None,
        release_info: None,
        genres: vec![],
        cast: vec![],
        directors: vec![],
        links: vec![],
        ratings: vec![],
    }
}

pub fn channel_meta(id: &str, name: &str, videos: Vec<Video>) -> MetaItem {
    MetaItem {
        preview: MetaItemPreview {
            id: id.to_owned(),
            r#type: "tv".to_owned(),
            name: name.to_owned(),
            ..MetaItemPreview::default()
        },
        videos,
    }
}

/// Live channels are stored as TEMPORARY items that are also `removed`
/// (`temp: true, removed: true`). The player resets a live item's progress
/// explicitly - `time_offset = 0` whatever duration the stream reported - so
/// recency is keyed on `last_watched`, not `time_offset`. `watched` false
/// mimics a channel that was never played (`last_watched = None`), which must
/// be excluded.
pub fn library_item(id: &str, r#type: &str, hour: u32, watched: bool) -> LibraryItem {
    LibraryItem {
        id: id.to_owned(),
        name: id.to_owned(),
        r#type: r#type.to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: true,
        temp: true,
        ctime: None,
        mtime: Utc.with_ymd_and_hms(2026, 7, 2, hour, 0, 0).unwrap(),
        state: LibraryItemState {
            time_offset: 0,
            duration: 0,
            last_watched: watched.then(|| Utc.with_ymd_and_hms(2026, 7, 2, hour, 0, 0).unwrap()),
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

pub fn epg_addon() -> Descriptor {
    Descriptor {
        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["tv".into()],
            resources: vec![META_RESOURCE_NAME.into()],
            id_prefixes: None,
            behavior_hints: ManifestBehaviorHints {
                epg_provider: true,
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

pub fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match &request {
        Request { url, method, .. }
            if url == "https://addon/meta/tv/pure%3Aaxn.json" && method == "GET" =>
        {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: channel_meta(
                    "pure:axn",
                    "AXN",
                    vec![
                        Video {
                            id: "pure:axn:2".to_owned(),
                            epg_info: Some(epg_info((12, 0), (13, 0))),
                            ..Video::default()
                        },
                        Video {
                            id: "pure:axn:1".to_owned(),
                            epg_info: Some(epg_info((11, 0), (12, 0))),
                            ..Video::default()
                        },
                        // no epg info - not a program show, must be dropped
                        Video {
                            id: "pure:axn:promo".to_owned(),
                            ..Video::default()
                        },
                    ],
                ),
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        Request { url, method, .. }
            if url == "https://addon/meta/tv/pure%3Aamc.json" && method == "GET" =>
        {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: channel_meta("pure:amc", "AMC", vec![]),
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}
