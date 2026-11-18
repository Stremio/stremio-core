use chrono::Duration;
use stremio_derive::Model;

use crate::{
    constants::{LIBRARY_RECENT_STORAGE_KEY, NOTIFICATIONS_STORAGE_KEY},
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Env, Runtime, RuntimeAction,
    },
    types::{
        library::{LibraryBucket, LibraryItem},
        notifications::{NotificationItem, NotificationsBucket},
    },
    unit_tests::{TestEnv, STORAGE},
};

#[test]
fn toggling_library_item_notifications_persists_the_preference_and_dismissal() {
    #[derive(Model, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let released = TestEnv::now() - Duration::days(1);
    let series = |id: &str| LibraryItem {
        id: id.to_owned(),
        name: id.to_owned(),
        r#type: "series".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: None,
        mtime: released,
        state: Default::default(),
        behavior_hints: Default::default(),
    };
    let notification = |id: &str| NotificationItem {
        meta_id: id.to_owned(),
        video_id: format!("{id}:1:1"),
        video_released: released,
    };
    let library = LibraryBucket::new(None, vec![series("tt1"), series("tt2")]);
    let notifications =
        NotificationsBucket::new::<TestEnv>(None, vec![notification("tt1"), notification("tt2")]);
    let mut expected_notifications = notifications.clone();
    expected_notifications.items.remove("tt1").unwrap();

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library,
                notifications,
                ..Default::default()
            },
        },
        vec![],
        1000,
    );

    for no_notif in [true, false] {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::ToggleLibraryItemNotifications(
                    "tt1".to_owned(),
                    no_notif,
                )),
            })
        });

        let model = runtime.model().unwrap();
        assert_eq!(model.ctx.library.items["tt1"].state.no_notif, no_notif);
        assert!(!model.ctx.library.items["tt2"].state.no_notif);
        assert_eq!(model.ctx.notifications, expected_notifications);

        let storage = STORAGE.read().unwrap();
        let stored_library: LibraryBucket =
            serde_json::from_str(&storage[LIBRARY_RECENT_STORAGE_KEY]).unwrap();
        let stored_notifications: NotificationsBucket =
            serde_json::from_str(&storage[NOTIFICATIONS_STORAGE_KEY]).unwrap();
        assert_eq!(stored_library, model.ctx.library);
        assert_eq!(stored_notifications, expected_notifications);
    }
}
