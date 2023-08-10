use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use assert_matches::assert_matches;
use chrono::{TimeZone, Utc};
use enclose::enclose;
use futures::future;
use serde::Deserialize;

use stremio_derive::Model;

use crate::{
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, ResourceResponse},
        library::{LibraryBucket, LibraryItem, LibraryItemState},
        notifications::{NotificationItem, NotificationsBucket},
        profile::Profile,
        resource::{MetaItemId, PosterShape, VideoId},
        streams::StreamsBucket,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, EVENTS, FETCH_HANDLER, STATES},
};

pub const DATA: &[u8] = include_bytes!("./data.json");

#[derive(Deserialize)]
struct TestData {
    network_requests: HashMap<String, ResourceResponse>,
    addons: Vec<Descriptor>,
    library_items: Vec<LibraryItem>,
    notification_items: Vec<NotificationItem>,
    result: HashMap<MetaItemId, HashMap<VideoId, NotificationItem>>,
}

#[test]
fn test_pull_notifications() {
    let tests = serde_json::from_slice::<Vec<TestData>>(DATA).unwrap();

    for test in tests {
        #[derive(Model, Clone, Debug)]
        #[model(TestEnv)]
        struct TestModel {
            ctx: Ctx,
        }
        let fetch_handler = enclose!((test.network_requests => network_requests) move |request: Request| -> TryEnvFuture<Box<dyn Any + Send>> {
            if let Some(result) = network_requests.get(&request.url) {
                return future::ok(Box::new(result.to_owned()) as Box<dyn Any + Send>).boxed_env();
            }

            return default_fetch_handler(request);
        });
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
        let (runtime, _rx) = Runtime::<TestEnv, _>::new(
            TestModel {
                ctx: Ctx::new(
                    Profile {
                        addons: test.addons,
                        ..Default::default()
                    },
                    LibraryBucket::new(None, test.library_items),
                    StreamsBucket::default(),
                    NotificationsBucket::new::<TestEnv>(None, test.notification_items),
                ),
            },
            vec![],
            1000,
        );
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::PullNotifications),
            })
        });

        pretty_assertions::assert_eq!(
            runtime.model().unwrap().ctx.notifications.items,
            test.result,
            "Notifications items match"
        );
    }
}

#[test]
fn test_dismiss_notification() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    ..Default::default()
                },
                LibraryBucket::new(
                    None,
                    vec![
                        LibraryItem {
                            id: "tt1".to_string(),
                            name: "Item 1".to_string(),
                            r#type: "series".to_string(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: false,
                            temp: false,
                            ctime: Some(Utc::now()),
                            mtime: Utc::now(),
                            state: LibraryItemState {
                                last_watched: Some(
                                    Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                                ),
                                time_watched: 40 * 60 * 60 * 1000,
                                time_offset: 15,
                                overall_time_watched: 140 * 60 * 60 * 1000,
                                times_watched: 2,
                                flagged_watched: 1,
                                duration: 55 * 60 * 60 * 1000,
                                video_id: Some("tt1:1".into()),
                                watched: None,
                                last_video_released: Some(Utc::now()),
                                no_notif: false,
                            },
                            behavior_hints: Default::default(),
                        },
                        LibraryItem {
                            id: "tt2".to_string(),
                            name: "Item 2".to_string(),
                            r#type: "series".to_string(),
                            poster: None,
                            poster_shape: PosterShape::Poster,
                            removed: false,
                            temp: false,
                            ctime: Some(Utc::now()),
                            mtime: Utc::now(),
                            state: LibraryItemState {
                                last_watched: Some(
                                    Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                                ),
                                time_watched: 40 * 60 * 60 * 1000,
                                time_offset: 15,
                                overall_time_watched: 140 * 60 * 60 * 1000,
                                times_watched: 2,
                                flagged_watched: 1,
                                duration: 55 * 60 * 60 * 1000,
                                video_id: Some("tt1:1".into()),
                                watched: None,
                                last_video_released: Some(Utc::now()),
                                no_notif: false,
                            },
                            behavior_hints: Default::default(),
                        },
                    ],
                ),
                StreamsBucket::default(),
                NotificationsBucket::new::<TestEnv>(
                    None,
                    vec![
                        NotificationItem {
                            meta_id: "tt1".to_string(),
                            video_id: "tt1:2".to_string(),
                            video_released: Utc::now(),
                        },
                        NotificationItem {
                            meta_id: "tt2".to_string(),
                            video_id: "tt2:10".to_string(),
                            video_released: Utc::now(),
                        },
                    ],
                ),
            ),
        },
        vec![],
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));

    let _env_mutex = TestEnv::reset().unwrap();

    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::DismissNotificationItem("tt1".into())),
            })
        }),
    );
    let events = EVENTS.read().unwrap();
    assert_eq!(events.len(), 2);

    let event_1 = events[0]
        .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
        .unwrap();

    let event_2 = events[1]
        .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
        .unwrap();

    assert_matches!(
        event_1,
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Ctx
    );
    assert_matches!(
        event_2,
        RuntimeEvent::CoreEvent(crate::runtime::msg::Event::NotificationsPushedToStorage {
            ids
        }) if ids == &vec!["tt2".to_string()]
    );

    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(2, states.len());

    // state 0 - we have both notifs.
    {
        let notification_items = &states[0].ctx.notifications.items;
        assert_eq!(
            2,
            notification_items.len(),
            "Should have both notifications"
        );
    }

    // state 1 - we have only the remaining notification after we've dismissed the other
    {
        let notification_items = &states[1].ctx.notifications.items;
        assert_eq!(
            1,
            notification_items.len(),
            "Should have single notification"
        );

        let notification_library_item = notification_items
            .get("tt2")
            .expect("Should have notification");
        assert!(
            notification_library_item.get("tt2:10").is_some(),
            "Should have notification"
        );
    }
}
