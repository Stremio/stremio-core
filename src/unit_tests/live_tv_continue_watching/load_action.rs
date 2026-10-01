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
        EnvFutureExt, Runtime, RuntimeAction,
    },
    types::{addon::ResourceResponse, library::LibraryBucket, profile::Profile},
    unit_tests::{
        live_tv_continue_watching::common::{channel_meta, epg_addon, fetch_handler, library_item},
        Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS,
    },
};

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
            (
                "tt123456".into(),
                library_item("tt123456", "movie", 13, true),
            ),
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

    let request_count = REQUESTS.read().unwrap().len();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::LiveTvContinueWatching),
        })
    });
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        request_count,
        "fresh schedules are reused"
    );
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 3, 0, 1, 0).unwrap();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::LiveTvContinueWatching),
        })
    });
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        request_count + 2,
        "expired schedules must be requested again"
    );

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

#[test]
fn live_channel_remains_removable_after_addon_uninstall() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_continue_watching: LiveTvContinueWatching,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(|request: Request| match &request {
        Request { url, .. } if url == "https://fallback/meta/tv/pure%3Aamc.json" => {
            future::ok(Box::new(ResourceResponse::Meta {
                meta: channel_meta("unavailable", "Unavailable channel", vec![]),
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => fetch_handler(request),
    });

    let mut fallback_addon = epg_addon();
    fallback_addon.transport_url = Url::parse("https://fallback/manifest.json").unwrap();
    fallback_addon.manifest.id = "fallback".to_owned();
    fallback_addon.manifest.behavior_hints.epg_provider = false;

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![epg_addon(), fallback_addon],
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: [("pure:amc".into(), library_item("pure:amc", "tv", 12, true))]
                        .into_iter()
                        .collect(),
                },
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
    assert_eq!(
        runtime.model().unwrap().live_tv_continue_watching.items[0]
            .channel
            .id,
        "pure:amc"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UninstallAddon(epg_addon())),
        });
    });

    let channel_id = runtime.model().unwrap().live_tv_continue_watching.items[0]
        .channel
        .id
        .clone();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RemoveFromLibrary(channel_id)),
        });
    });

    let model = runtime.model().unwrap();
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "the card remains dismissible when a fallback addon returns a different metadata ID"
    );
    assert!(!model.ctx.library.items["pure:amc"].temp);
}
