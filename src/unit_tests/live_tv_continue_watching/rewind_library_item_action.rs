use chrono::{TimeZone, Utc};
use stremio_derive::Model;

use crate::{
    models::{ctx::Ctx, live_tv_continue_watching::LiveTvContinueWatching},
    runtime::{
        msg::{Action, ActionCtx, ActionLoad},
        Runtime, RuntimeAction, RuntimeEvent,
    },
    types::{
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
    },
    unit_tests::{
        live_tv_continue_watching::common::{epg_addon, fetch_handler, library_item},
        TestEnv, FETCH_HANDLER, NOW,
    },
};

const CHANNEL_ID: &str = "pure:axn";

#[derive(Model, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    live_tv_continue_watching: LiveTvContinueWatching,
}

/// A channel that was played but never explicitly saved: temporary and
/// `removed`, which is how the player stores live channels.
fn temp_channel() -> LibraryItem {
    library_item(CHANNEL_ID, "tv", 11, true)
}

/// The same channel after the user explicitly added it to their library.
fn saved_channel() -> LibraryItem {
    LibraryItem {
        removed: false,
        temp: false,
        ..temp_channel()
    }
}

/// Boots the row with `channel` in the library and loads it, asserting the
/// channel is on screen before the dismiss under test.
fn loaded_row(
    channel: LibraryItem,
) -> (
    Runtime<TestEnv, TestModel>,
    futures::channel::mpsc::Receiver<RuntimeEvent<TestEnv, TestModel>>,
) {
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2026, 7, 2, 11, 30, 0).unwrap();

    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![epg_addon()],
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: None,
                    items: [(CHANNEL_ID.into(), channel)].into_iter().collect(),
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
        runtime
            .model()
            .unwrap()
            .live_tv_continue_watching
            .items
            .len(),
        1,
        "the watched channel is in the row"
    );

    (runtime, rx)
}

fn dispatch(runtime: &Runtime<TestEnv, TestModel>, action: ActionCtx) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(action),
        });
    });
}

#[test]
fn rewind_dismisses_a_temporary_live_channel() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = loaded_row(temp_channel());

    dispatch(
        &runtime,
        ActionCtx::RewindLibraryItem(CHANNEL_ID.to_owned()),
    );

    let model = runtime.model().unwrap();
    let library_item = model
        .ctx
        .library
        .items
        .get(CHANNEL_ID)
        .expect("library item should still exist");
    assert_eq!(
        library_item.state.last_watched, None,
        "rewinding a live channel clears last_watched"
    );
    assert!(
        library_item.removed && library_item.temp,
        "rewind must not touch removed/temp: {:?}",
        (library_item.removed, library_item.temp)
    );
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "the dismissed channel is removed from the row"
    );
}

/// The reason the dismiss button rewinds instead of removing: a channel the
/// user saved to their library must stay saved after being dismissed from the
/// Continue Watching row.
#[test]
fn rewind_keeps_a_saved_live_channel_saved() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = loaded_row(saved_channel());

    dispatch(
        &runtime,
        ActionCtx::RewindLibraryItem(CHANNEL_ID.to_owned()),
    );

    let model = runtime.model().unwrap();
    let library_item = model
        .ctx
        .library
        .items
        .get(CHANNEL_ID)
        .expect("library item should still exist");
    assert_eq!(
        library_item.state.last_watched, None,
        "rewinding a live channel clears last_watched"
    );
    assert!(
        !library_item.removed,
        "dismissing must not unsave the channel"
    );
    assert!(
        !library_item.temp,
        "dismissing must not turn a saved channel back into a temporary one"
    );
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "the dismissed channel is removed from the row"
    );
}

/// The contrast the fix is about. `RemoveFromLibrary` also keeps the record -
/// it is never erased from the bucket - but it flips `removed`/`temp`, which
/// unsaves a channel the user had added on purpose.
#[test]
fn remove_from_library_keeps_the_record_but_unsaves_the_channel() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = loaded_row(saved_channel());

    dispatch(
        &runtime,
        ActionCtx::RemoveFromLibrary(CHANNEL_ID.to_owned()),
    );

    let model = runtime.model().unwrap();
    let library_item = model
        .ctx
        .library
        .items
        .get(CHANNEL_ID)
        .expect("RemoveFromLibrary marks the record removed, it does not erase it");
    assert!(
        library_item.removed && !library_item.temp,
        "RemoveFromLibrary unsaves the channel: {:?}",
        (library_item.removed, library_item.temp)
    );
    assert!(
        library_item.state.last_watched.is_some(),
        "RemoveFromLibrary leaves last_watched alone - only the removed/temp \
         flags drop it from the row"
    );
    assert!(
        model.live_tv_continue_watching.items.is_empty(),
        "the channel is gone from the row either way"
    );
}
