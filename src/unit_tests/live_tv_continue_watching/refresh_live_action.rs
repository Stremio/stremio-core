use chrono::{Duration, TimeZone, Utc};
use stremio_derive::Model;

use crate::{
    models::{ctx::Ctx, live_tv_continue_watching::LiveTvContinueWatching},
    runtime::{
        msg::{Action, ActionLiveTvContinueWatching, ActionLoad},
        Runtime, RuntimeAction,
    },
    types::{library::LibraryBucket, profile::Profile},
    unit_tests::{
        live_tv_continue_watching::common::{epg_addon, fetch_handler, library_item},
        TestEnv, FETCH_HANDLER, NOW, REQUESTS,
    },
};

#[test]
fn live_tv_continue_watching_refresh_live() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        live_tv_continue_watching: LiveTvContinueWatching,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 11, 30, 0).unwrap();

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![epg_addon()],
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: [("pure:axn".into(), library_item("pure:axn", "tv", 11, true))]
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
    let dispatch = |action: Action| {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action,
            });
        });
    };
    let refresh = || Action::LiveTvContinueWatching(ActionLiveTvContinueWatching::RefreshLive);

    dispatch(Action::Load(ActionLoad::LiveTvContinueWatching));
    assert_eq!(REQUESTS.read().unwrap().len(), 1);
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .live_tv_continue_watching
            .items
            .len(),
        1
    );

    dispatch(refresh());
    *NOW.write().unwrap() += Duration::minutes(14);
    dispatch(refresh());
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "a fresh schedule is not refetched"
    );

    *NOW.write().unwrap() += Duration::minutes(1);
    dispatch(refresh());
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        2,
        "a refresh refetches the stale schedule"
    );

    dispatch(Action::Unload);
    *NOW.write().unwrap() += Duration::minutes(15);
    dispatch(refresh());
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        2,
        "an unloaded model ignores refreshes"
    );
}
