use crate::constants::LIBRARY_RECENT_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Runtime, RuntimeAction};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::{LibraryBucket, LibraryItem, LibraryItemState};
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{TestEnv, NOW, STORAGE};
use chrono::{TimeZone, Utc};
use stremio_derive::Model;

fn meta_preview() -> MetaItemPreview {
    MetaItemPreview {
        id: "tt123".into(),
        r#type: "movie".to_owned(),
        name: "Test Movie".to_owned(),
        poster: None,
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    }
}

fn test_runtime(library: LibraryBucket) -> (Runtime<TestEnv, TestModel>, impl std::any::Any) {
    Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile::default(),
                library,
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    )
}

#[derive(Model, Clone, Default)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
}

#[test]
fn meta_item_mark_as_watched_missing_item_creates_temp() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let (runtime, _rx) = test_runtime(LibraryBucket::default());
    assert!(
        runtime.model().unwrap().ctx.library.items.is_empty(),
        "Library starts empty"
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::MetaItemMarkAsWatched {
                meta_item: meta_preview(),
                is_watched: true,
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert_eq!(model.ctx.library.items.len(), 1, "One item created");
    let item = model.ctx.library.items.get("tt123").expect("Item exists");
    assert_eq!(item.state.times_watched, 1, "times_watched is 1");
    assert!(item.state.last_watched.is_some(), "last_watched is set");
    assert!(item.temp, "Item is temporary");
    assert!(item.removed, "Item is marked as removed");
}

#[test]
fn meta_item_mark_as_watched_existing_item_increments() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let existing = LibraryItem {
        id: "tt123".into(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: LibraryItemState {
            times_watched: 2,
            last_watched: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
            ..Default::default()
        },
        name: "Test Movie".to_owned(),
        r#type: "movie".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let (runtime, _rx) = test_runtime(LibraryBucket {
        uid: None,
        items: vec![("tt123".into(), existing)].into_iter().collect(),
    });
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::MetaItemMarkAsWatched {
                meta_item: meta_preview(),
                is_watched: true,
            }),
        })
    });
    let model = runtime.model().unwrap();
    let item = model.ctx.library.items.get("tt123").expect("Item exists");
    assert_eq!(
        item.state.times_watched, 3,
        "times_watched incremented to 3"
    );
    assert!(!item.temp, "Item remains non-temp");
    assert!(!item.removed, "Item remains non-removed");
}

#[test]
fn meta_item_unwatch_missing_item_is_noop() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let (runtime, _rx) = test_runtime(LibraryBucket::default());
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::MetaItemMarkAsWatched {
                meta_item: meta_preview(),
                is_watched: false,
            }),
        })
    });
    assert!(
        runtime.model().unwrap().ctx.library.items.is_empty(),
        "No ghost item created for unwatch on missing item"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .is_none(),
        "Nothing persisted to storage"
    );
}

#[test]
fn meta_item_unwatch_existing_item_resets() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let existing = LibraryItem {
        id: "tt123".into(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: LibraryItemState {
            times_watched: 5,
            last_watched: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
            ..Default::default()
        },
        name: "Test Movie".to_owned(),
        r#type: "movie".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let (runtime, _rx) = test_runtime(LibraryBucket {
        uid: None,
        items: vec![("tt123".into(), existing)].into_iter().collect(),
    });
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::MetaItemMarkAsWatched {
                meta_item: meta_preview(),
                is_watched: false,
            }),
        })
    });
    let model = runtime.model().unwrap();
    let item = model
        .ctx
        .library
        .items
        .get("tt123")
        .expect("Item still exists");
    assert_eq!(item.state.times_watched, 0, "times_watched reset to 0");
}
