use std::any::Any;

use chrono::{TimeZone, Utc};
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::META_RESOURCE_NAME,
    models::{ctx::Ctx, live_tv_continue_watching::LiveTvContinueWatching},
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestBehaviorHints, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video, VideoEpgInfo},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW},
};

fn epg_info(start: (u32, u32), end: (u32, u32)) -> VideoEpgInfo {
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
    }
}

fn channel_meta(id: &str, name: &str, videos: Vec<Video>) -> MetaItem {
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
/// (`temp: true, removed: true`) and, after the player unloads, `time_offset =
/// 0` (live streams report `duration == 0`, so the player's credits-threshold
/// reset always zeroes it). Recency is therefore keyed on `last_watched`, not
/// `time_offset`. `watched` false mimics a channel that was never played
/// (`last_watched = None`), which must be excluded.
fn library_item(id: &str, r#type: &str, hour: u32, watched: bool) -> LibraryItem {
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
            // 0 mirrors reality: the player zeroes a live channel's
            // time_offset on Unload. Included channels must survive that.
            time_offset: 0,
            duration: 0,
            last_watched: watched
                .then(|| Utc.with_ymd_and_hms(2026, 7, 2, hour, 0, 0).unwrap()),
            ..Default::default()
        },
        behavior_hints: Default::default(),
    }
}

fn epg_addon() -> Descriptor {
    Descriptor {
        transport_url: Url::parse("https://addon/manifest.json").unwrap(),
        flags: Default::default(),
        manifest: Manifest {
            id: "addon".to_owned(),
            types: vec!["tv".into()],
            resources: vec![META_RESOURCE_NAME.into()],
            id_prefixes: Some(vec!["pure:".to_owned()]),
            behavior_hints: ManifestBehaviorHints {
                epg_provider: true,
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
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

#[test]
fn live_tv_continue_watching() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_continue_watching: LiveTvContinueWatching,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 12, 30, 0).unwrap();

    let profile = Profile {
        addons: vec![epg_addon()],
        ..Default::default()
    };
    let library = LibraryBucket {
        uid: None,
        items: vec![
            // regular item (not an epgProvider channel) - excluded
            ("tt123456".into(), library_item("tt123456", "movie", 13, true)),
            // watched channels; amc is more recent than axn. Both sit at
            // time_offset 0 (post-Unload) yet must still appear - this is the
            // regression the fix guards against.
            ("pure:axn".into(), library_item("pure:axn", "tv", 11, true)),
            ("pure:amc".into(), library_item("pure:amc", "tv", 12, true)),
            // a never-played channel (last_watched None) - excluded, no fetch
            ("pure:old".into(), library_item("pure:old", "tv", 9, false)),
        ]
        .into_iter()
        .collect(),
    };

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile,
                library,
                ..Default::default()
            },
            live_tv_continue_watching: Default::default(),
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::LiveTvContinueWatching),
        });
    });

    {
        let model = runtime.model().unwrap();
        let live_tv = &model.live_tv_continue_watching;
        assert_eq!(
            live_tv
                .items
                .iter()
                .map(|item| item.channel.id.as_str())
                .collect::<Vec<_>>(),
            vec!["pure:amc", "pure:axn"],
            "only watched epgProvider channels, most recent first; both are at \
             time_offset 0 (post-Unload) yet still included, while the regular \
             item and the never-played (last_watched None) channel are excluded"
        );
        assert_eq!(
            live_tv.items[0].channel.name, "AMC",
            "the channel preview comes from the fetched meta"
        );
        assert!(
            live_tv.items[0].shows.is_empty(),
            "a channel with no program shows still gets a card, with no shows"
        );
        assert_eq!(
            live_tv.items[1]
                .shows
                .iter()
                .map(|show| show.id.as_str())
                .collect::<Vec<_>>(),
            vec!["pure:axn:2", "pure:axn:1"],
            "only program shows (with epg_info) are kept; order is left to the frontend"
        );
        assert_eq!(
            live_tv.items[1].request.path.resource, META_RESOURCE_NAME,
            "the item carries the channel's meta request for deep links"
        );
    }

    // uninstalling the epgProvider addon empties the row (recompute on
    // ProfileChanged)
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UninstallAddon(epg_addon())),
        });
    });

    let model = runtime.model().unwrap();
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "the row empties once the epgProvider addon is uninstalled"
    );
}
