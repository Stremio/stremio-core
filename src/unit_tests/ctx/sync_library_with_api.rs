use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    constants::{LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_STORAGE_KEY},
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        api::{APIResult, LibraryItemModified, LibraryItemsResponse, SuccessResponse},
        events::DismissedEventsBucket,
        library::{LibraryBucket, LibraryItem},
        notifications::NotificationsBucket,
        profile::{Auth, AuthKey, GDPRConsent, Profile, User},
        resource::MetaItemPreview,
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
        True,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE},
};

use chrono::prelude::TimeZone;
use chrono::{Duration, Utc};
use futures::future;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use stremio_derive::Model;

#[test]
fn actionctx_synclibrarywithapi() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
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
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::SyncLibraryWithAPI),
        })
    });
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_synclibrarywithapi_with_user() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    static REMOTE_ONLY_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id1".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static LOCAL_NEWER_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id2".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static REMOTE_NEWER_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id3".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static LOCAL_ONLY_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id4".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static LOCAL_OLD_REMOVED_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id5".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: true,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc::now() - Duration::days(367),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static LOCAL_NEW_REMOVED_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id6".to_owned(),
        r#type: "type".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: true,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc::now() - Duration::days(3),
        state: Default::default(),
        behavior_hints: Default::default(),
    });
    static LOCAL_OTHER_TYPE_ITEM: Lazy<LibraryItem> = Lazy::new(|| LibraryItem {
        id: "id7".to_owned(),
        r#type: "other".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc::now(),
        state: Default::default(),
        behavior_hints: Default::default(),
    });

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match &request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreMeta"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}" =>
            {
                future::ok(Box::new(APIResult::Ok(vec![
                    LibraryItemModified(
                        REMOTE_ONLY_ITEM.id.to_owned(),
                        REMOTE_ONLY_ITEM.mtime.to_owned(),
                    ),
                    LibraryItemModified(
                        LOCAL_NEWER_ITEM.id.to_owned(),
                        LOCAL_NEWER_ITEM.mtime - Duration::days(1),
                    ),
                    LibraryItemModified(
                        REMOTE_NEWER_ITEM.id.to_owned(),
                        REMOTE_NEWER_ITEM.mtime.to_owned(),
                    ),
                ])) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut" && method == "POST" => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Body {
                    auth_key: AuthKey,
                    collection: String,
                    changes: Vec<LibraryItem>,
                }
                match serde_json::from_str::<Body>(body) {
                    Result::Ok(body)
                        if body.auth_key == AuthKey("auth_key".to_owned())
                            && body.collection == "libraryItem"
                            && body.changes.len() == 3
                            && body.changes.contains(&LOCAL_NEWER_ITEM)
                            && body.changes.contains(&LOCAL_ONLY_ITEM)
                            && body.changes.contains(&LOCAL_NEW_REMOVED_ITEM) =>
                    {
                        future::ok(
                            Box::new(APIResult::Ok(SuccessResponse { success: True {} }))
                                as Box<dyn Any + Send>,
                        )
                        .boxed_env()
                    }
                    _ => default_fetch_handler(request),
                }
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet" && method == "POST" => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Body {
                    auth_key: AuthKey,
                    collection: String,
                    all: bool,
                    ids: Vec<String>,
                }
                match serde_json::from_str::<Body>(body) {
                    Result::Ok(body)
                        if body.auth_key == AuthKey("auth_key".to_owned())
                            && body.collection == "libraryItem"
                            && !body.all
                            && body.ids.len() == 2
                            && body.ids.contains(&REMOTE_ONLY_ITEM.id)
                            && body.ids.contains(&REMOTE_NEWER_ITEM.id) =>
                    {
                        future::ok(Box::new(APIResult::Ok(LibraryItemsResponse(vec![
                            REMOTE_ONLY_ITEM.to_owned(),
                            REMOTE_NEWER_ITEM.to_owned(),
                        ]))) as Box<dyn Any + Send>)
                        .boxed_env()
                    }
                    _ => default_fetch_handler(request),
                }
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".into(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            apple_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            trakt: None,
                            premium_expire: None,
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: Some("tests".to_owned()),
                            },
                            ..Default::default()
                        },
                    }),
                    ..Default::default()
                },
                LibraryBucket {
                    uid: Some("user_id".into()),
                    items: vec![
                        (LOCAL_ONLY_ITEM.id.to_owned(), LOCAL_ONLY_ITEM.to_owned()),
                        (LOCAL_NEWER_ITEM.id.to_owned(), LOCAL_NEWER_ITEM.to_owned()),
                        (
                            REMOTE_NEWER_ITEM.id.to_owned(),
                            LibraryItem {
                                mtime: REMOTE_NEWER_ITEM.mtime - Duration::days(1),
                                ..REMOTE_NEWER_ITEM.to_owned()
                            },
                        ),
                        (
                            LOCAL_OLD_REMOVED_ITEM.id.to_owned(),
                            LOCAL_OLD_REMOVED_ITEM.to_owned(),
                        ),
                        (
                            LOCAL_NEW_REMOVED_ITEM.id.to_owned(),
                            LOCAL_NEW_REMOVED_ITEM.to_owned(),
                        ),
                        (
                            LOCAL_OTHER_TYPE_ITEM.id.to_owned(),
                            LOCAL_OTHER_TYPE_ITEM.to_owned(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::SyncLibraryWithAPI),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        LibraryBucket {
            uid: Some("user_id".into()),
            items: vec![
                (LOCAL_ONLY_ITEM.id.to_owned(), LOCAL_ONLY_ITEM.to_owned()),
                (LOCAL_NEWER_ITEM.id.to_owned(), LOCAL_NEWER_ITEM.to_owned()),
                (
                    REMOTE_NEWER_ITEM.id.to_owned(),
                    REMOTE_NEWER_ITEM.to_owned()
                ),
                (REMOTE_ONLY_ITEM.id.to_owned(), REMOTE_ONLY_ITEM.to_owned()),
                (
                    LOCAL_OLD_REMOVED_ITEM.id.to_owned(),
                    LOCAL_OLD_REMOVED_ITEM.to_owned()
                ),
                (
                    LOCAL_NEW_REMOVED_ITEM.id.to_owned(),
                    LOCAL_NEW_REMOVED_ITEM.to_owned()
                ),
                (
                    LOCAL_OTHER_TYPE_ITEM.id.to_owned(),
                    LOCAL_OTHER_TYPE_ITEM.to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map(|data| serde_json::from_str::<LibraryBucket>(data).unwrap()),
        Some(LibraryBucket::new(
            Some("user_id".into()),
            vec![
                REMOTE_ONLY_ITEM.to_owned(),
                LOCAL_ONLY_ITEM.to_owned(),
                REMOTE_NEWER_ITEM.to_owned(),
                LOCAL_NEWER_ITEM.to_owned(),
                LOCAL_OLD_REMOVED_ITEM.to_owned(),
                LOCAL_NEW_REMOVED_ITEM.to_owned(),
                LOCAL_OTHER_TYPE_ITEM.to_owned(),
            ]
        )),
        "Library recent slot updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().url,
        "https://api.strem.io/api/datastoreMeta".to_owned(),
        "datastoreMeta request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(1).unwrap().url,
        "https://api.strem.io/api/datastorePut".to_owned(),
        "datastorePut request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(2).unwrap().url,
        "https://api.strem.io/api/datastoreGet".to_owned(),
        "datastoreGet request has been sent"
    );
}

#[test]
fn actionctx_synclibrarywithapi_with_user_empty_library() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match &request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreMeta"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}" =>
            {
                future::ok(Box::new(APIResult::Ok(Vec::<LibraryItemModified>::new()))
                    as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".into(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            apple_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            trakt: None,
                            premium_expire: None,
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: Some("tests".to_owned()),
                            },
                            ..Default::default()
                        },
                    }),
                    ..Default::default()
                },
                LibraryBucket::default(),
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::SyncLibraryWithAPI),
        })
    });
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().url,
        "https://api.strem.io/api/datastoreMeta".to_owned(),
        "datastoreMeta request has been sent"
    );
}

#[test]
fn library_addition_syncs_between_runtimes() {
    #[derive(Model, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    const ITEM_ID: &str = "tt1234567";

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let server_items = Arc::new(Mutex::new(HashMap::<String, LibraryItem>::new()));
    *FETCH_HANDLER.write().unwrap() = Box::new(move |request: Request| {
        let server_items = Arc::clone(&server_items);
        future::lazy(move |_| {
            let body: Value = serde_json::from_str(&request.body).unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(body["collection"], LIBRARY_COLLECTION_NAME);

            let mut items = server_items.lock().unwrap();
            let response: Box<dyn Any + Send> = match request.url.as_str() {
                "https://api.strem.io/api/datastorePut" => {
                    assert_eq!(body["authKey"], "source_key");
                    let changes: Vec<LibraryItem> =
                        serde_json::from_value(body["changes"].to_owned()).unwrap();
                    items.extend(changes.into_iter().map(|item| (item.id.to_owned(), item)));
                    Box::new(APIResult::Ok(SuccessResponse { success: True }))
                }
                "https://api.strem.io/api/datastoreMeta" => {
                    assert_eq!(body["authKey"], "destination_key");
                    let metadata: Vec<_> = items
                        .values()
                        .map(|item| LibraryItemModified(item.id.to_owned(), item.mtime))
                        .collect();
                    Box::new(APIResult::Ok(metadata))
                }
                "https://api.strem.io/api/datastoreGet" => {
                    assert_eq!(body["authKey"], "destination_key");
                    assert_eq!(body["all"], false);
                    let ids: Vec<String> = serde_json::from_value(body["ids"].to_owned()).unwrap();
                    assert_eq!(ids, [ITEM_ID]);
                    let requested = ids.iter().map(|id| items[id].to_owned()).collect();
                    Box::new(APIResult::Ok(LibraryItemsResponse(requested)))
                }
                url => panic!("Unexpected datastore endpoint: {url}"),
            };
            Ok(response)
        })
        .boxed_env()
    });

    let new_runtime = |auth_key: &str| {
        let profile = Profile {
            auth: Some(Auth {
                key: AuthKey(auth_key.to_owned()),
                user: User {
                    id: "user_id".into(),
                    ..Default::default()
                },
            }),
            addons: vec![],
            ..Default::default()
        };
        let ctx = Ctx {
            library: LibraryBucket::new(profile.uid(), vec![]),
            profile,
            ..Default::default()
        };
        Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000)
    };
    let (source, _source_events) = new_runtime("source_key");
    let (destination, _destination_events) = new_runtime("destination_key");
    let meta = MetaItemPreview {
        id: ITEM_ID.into(),
        r#type: "movie".into(),
        name: "Example movie".into(),
        ..Default::default()
    };

    TestEnv::run(|| {
        source.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::AddToLibrary(meta)),
        });
    });
    let source_item = source.model().unwrap().ctx.library.items[ITEM_ID].to_owned();
    assert!(destination.model().unwrap().ctx.library.items.is_empty());

    TestEnv::run(|| {
        destination.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::SyncLibraryWithAPI),
        });
    });
    assert_eq!(
        destination.model().unwrap().ctx.library.items,
        HashMap::from([(ITEM_ID.to_owned(), source_item)]),
        "The destination should sync the item uploaded by the source runtime"
    );
}
