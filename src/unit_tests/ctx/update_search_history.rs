use stremio_derive::Model;

use crate::{
    constants::SEARCH_HISTORY_STORAGE_KEY,
    models::{
        catalogs_with_extra::{CatalogsWithExtra, Selected},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad},
        Env, Runtime, RuntimeAction,
    },
    types::{
        addon::ExtraValue, library::LibraryBucket, notifications::NotificationsBucket,
        profile::Profile, search_history::SearchHistoryBucket, streams::StreamsBucket,
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

    let search_history = SearchHistoryBucket::default();

    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
    );

    let catalogs_with_extra = CatalogsWithExtra::default();

    STORAGE.write().unwrap().insert(
        SEARCH_HISTORY_STORAGE_KEY.to_owned(),
        serde_json::to_string(&search_history).unwrap(),
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
                    value: "superman".to_owned(),
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
                    .get(query)
                    .is_some()
            }),
        "Should have stored updated search history"
    );
}
