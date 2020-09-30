use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Effects, Env, EnvFuture, Runtime};
use crate::types::api::{APIResult, SuccessResponse, True};
use crate::types::library::{LibraryBucket, LibraryItem, LibraryItemState};
use crate::types::profile::{Auth, GDPRConsent, Profile, User};
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use futures::{future, FutureExt};
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_rewindlibraryitem() {
    #[derive(Model, Default)]
    struct TestModel {
        ctx: Ctx<TestEnv>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-02T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>).boxed_local()
            }
            _ => default_fetch_handler(request),
        }
    }
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState {
            time_offset: 10,
            ..LibraryItemState::default()
        },
        name: "name".to_owned(),
        type_name: "type_name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let library_item_rewinded = LibraryItem {
        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState {
            time_offset: 0,
            ..LibraryItemState::default()
        },
        ..library_item.to_owned()
    };
    TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0);
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(
            Some("id".to_owned()),
            vec![library_item.to_owned()],
        ))
        .unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: "tests".to_owned(),
                            },
                        },
                    }),
                    ..Default::default()
                },
                library: LibraryBucket {
                    uid: Some("id".to_owned()),
                    items: vec![("id".to_owned(), library_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
        },
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(Action::Ctx(ActionCtx::RewindLibraryItem(
            library_item.id.to_owned(),
        )))
    });
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&library_item.id),
        Some(&library_item_rewinded),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(&data).unwrap()
                    == LibraryBucket::new(Some("id".to_owned()), vec![library_item_rewinded])
            }),
        "Library recent slot updated successfully in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().url.to_owned(),
        "https://api.strem.io/api/datastorePut".to_owned(),
        "datastorePut request has been sent"
    );
}

#[test]
fn actionctx_rewindlibraryitem_not_added() {
    #[derive(Model, Default)]
    struct TestModel {
        ctx: Ctx<TestEnv>,
    }
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState {
            time_offset: 10,
            ..LibraryItemState::default()
        },
        name: "name".to_owned(),
        type_name: "type_name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    TestEnv::reset();
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(None, vec![library_item.to_owned()])).unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                library: LibraryBucket {
                    uid: None,
                    items: vec![("id".to_owned(), library_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                ..Default::default()
            },
        },
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| runtime.dispatch(Action::Ctx(ActionCtx::RewindLibraryItem("id2".to_owned()))));
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&library_item.id),
        Some(&library_item),
        "Library not updated in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(&data).unwrap()
                    == LibraryBucket::new(None, vec![library_item])
            }),
        "Library recent slot not updated in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
