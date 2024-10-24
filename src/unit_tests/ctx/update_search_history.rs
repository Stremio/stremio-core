use stremio_derive::Model;

use crate::{
    constants::SEARCH_HISTORY_STORAGE_KEY,
    models::{
        catalogs_with_extra::{CatalogsWithExtra, Selected},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        Env, Runtime, RuntimeAction,
    },
    types::{
        addon::ExtraValue, events::DismissedEventsBucket, library::LibraryBucket,
        notifications::NotificationsBucket, profile::Profile, search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket, streams::StreamsBucket,
    },
    unit_tests::{TestEnv, STORAGE},
};

#[test]
fn test_search_history_update() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        catalogs_with_extra: CatalogsWithExtra,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );

    let catalogs_with_extra = CatalogsWithExtra::default();

    STORAGE.write().unwrap().insert(
        SEARCH_HISTORY_STORAGE_KEY.to_owned(),
        serde_json::to_string(&ctx.search_history).unwrap(),
    );

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx,
            catalogs_with_extra,
        },
        vec![],
        1000,
    );

    let query = "superman";
    let date = TestEnv::now();

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::CatalogsWithExtra(Selected {
                r#type: None,
                extra: vec![ExtraValue {
                    name: "search".to_owned(),
                    value: query.to_owned(),
                }],
            })),
        })
    });

    assert_eq!(
        runtime.model().unwrap().ctx.search_history.items.get(query),
        Some(date).as_ref(),
        "Should have updated search history"
    );

    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(SEARCH_HISTORY_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<SearchHistoryBucket>(data)
                    .unwrap()
                    .items
                    .contains_key(query)
            }),
        "Should have stored updated search history"
    );
}

#[test]
fn test_search_history_clear_items() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        catalogs_with_extra: CatalogsWithExtra,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );

    let catalogs_with_extra = CatalogsWithExtra::default();

    STORAGE.write().unwrap().insert(
        SEARCH_HISTORY_STORAGE_KEY.to_owned(),
        serde_json::to_string(&ctx.search_history).unwrap(),
    );

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx,
            catalogs_with_extra,
        },
        vec![],
        1000,
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Load(ActionLoad::CatalogsWithExtra(Selected {
                r#type: None,
                extra: vec![ExtraValue {
                    name: "search".to_owned(),
                    value: "superman".to_owned(),
                }],
            })),
        })
    });

    assert!(
        !runtime.model().unwrap().ctx.search_history.items.is_empty(),
        "Should have updated search history"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::ClearSearchHistory),
        })
    });

    assert!(
        runtime.model().unwrap().ctx.search_history.items.is_empty(),
        "Should have cleared search history"
    );
}
