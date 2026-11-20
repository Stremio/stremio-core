use chrono::Duration;
use stremio_derive::Model;

use crate::{
    constants::{LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY},
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Env, Runtime, RuntimeAction,
    },
    types::library::{LibraryBucket, LibraryItem, LibraryItemState},
    unit_tests::{TestEnv, STORAGE},
};

#[test]
fn rewinding_a_recent_library_item_only_updates_recent_storage() {
    #[derive(Model, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let now = TestEnv::now();
    let library_item = |index: usize| LibraryItem {
        id: format!("tt{index}"),
        name: format!("Movie {index}"),
        r#type: "movie".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: now - Duration::seconds(index as i64 + 1),
        state: LibraryItemState {
            time_offset: 10,
            ..Default::default()
        },
        behavior_hints: Default::default(),
    };
    let mut recent =
        LibraryBucket::new(None, (0..LIBRARY_RECENT_COUNT).map(library_item).collect());
    let older = LibraryBucket::new(None, vec![library_item(LIBRARY_RECENT_COUNT)]);
    // Leading whitespace makes a rewrite of the older bucket observable.
    let stored_older = format!("\n{}", serde_json::to_string(&older).unwrap());
    STORAGE.write().unwrap().extend([
        (
            LIBRARY_RECENT_STORAGE_KEY.to_owned(),
            serde_json::to_string(&recent).unwrap(),
        ),
        (LIBRARY_STORAGE_KEY.to_owned(), stored_older.clone()),
    ]);

    let mut library = recent.clone();
    library.merge_bucket(older);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library,
                ..Default::default()
            },
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RewindLibraryItem("tt0".to_owned())),
        });
    });

    let expected = recent.items.get_mut("tt0").unwrap();
    expected.state.time_offset = 0;
    expected.mtime = now;
    assert_eq!(&runtime.model().unwrap().ctx.library.items["tt0"], expected);

    let storage = STORAGE.read().unwrap();
    let stored_recent: LibraryBucket =
        serde_json::from_str(&storage[LIBRARY_RECENT_STORAGE_KEY]).unwrap();
    assert_eq!(stored_recent, recent);
    assert_eq!(
        storage[LIBRARY_STORAGE_KEY], stored_older,
        "rewinding a recent item should not rewrite the older bucket"
    );
}
