use std::any::Any;

use crate::constants::LIBRARY_RECENT_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, LibraryItemModified, LibraryItemsResponse, SuccessResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::types::True;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE,
};

use chrono::prelude::TimeZone;
use chrono::{Duration, Utc};
use futures::future;
use once_cell::sync::Lazy;
use serde::Deserialize;
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
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
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
                        },
                    }),
                    ..Default::default()
                },
                LibraryBucket {
                    uid: Some("user_id".to_owned()),
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
            uid: Some("user_id".to_string()),
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
            Some("user_id".to_owned()),
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
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
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
