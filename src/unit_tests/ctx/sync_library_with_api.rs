use crate::constants::LIBRARY_RECENT_STORAGE_KEY;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, LibItemModified, SuccessResponse, True};
use crate::types::library::{LibBucket, LibItem, LibItemState};
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use chrono::prelude::TimeZone;
use chrono::{Duration, Utc};
use futures::future;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_synclibrarywithapi() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    Env::reset();
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI))));
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_synclibrarywithapi_with_user() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    lazy_static! {
        static ref REMOTE_ONLY_ITEM: LibItem = LibItem {
            id: "id1".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: false,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: Default::default(),
            behavior_hints: Default::default(),
        };
        static ref LOCAL_NEWER_ITEM: LibItem = LibItem {
            id: "id2".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: false,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: Default::default(),
            behavior_hints: Default::default(),
        };
        static ref REMOTE_NEWER_ITEM: LibItem = LibItem {
            id: "id3".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: false,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: Default::default(),
            behavior_hints: Default::default(),
        };
        static ref LOCAL_ONLY_ITEM: LibItem = LibItem {
            id: "id4".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: false,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: Default::default(),
            behavior_hints: Default::default(),
        };
        static ref LOCAL_NOT_WATCHED_ITEM: LibItem = LibItem {
            id: "id5".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: true,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: Default::default(),
            behavior_hints: Default::default(),
        };
        static ref LOCAL_WATCHED_ITEM: LibItem = LibItem {
            id: "id6".to_owned(),
            type_name: "type_name".to_owned(),
            name: "name".to_owned(),
            poster: None,
            poster_shape: Default::default(),
            removed: true,
            temp: false,
            ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            state: LibItemState {
                overall_time_watched: 60001,
                ..LibItemState::default()
            },
            behavior_hints: Default::default(),
        };
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match &request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreMeta"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: vec![
                        LibItemModified(
                            REMOTE_ONLY_ITEM.id.to_owned(),
                            REMOTE_ONLY_ITEM.mtime.to_owned(),
                        ),
                        LibItemModified(
                            LOCAL_NEWER_ITEM.id.to_owned(),
                            LOCAL_NEWER_ITEM.mtime - Duration::days(1),
                        ),
                        LibItemModified(
                            REMOTE_NEWER_ITEM.id.to_owned(),
                            REMOTE_NEWER_ITEM.mtime.to_owned(),
                        ),
                    ],
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut" && method == "POST" => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Body {
                    auth_key: AuthKey,
                    collection: String,
                    changes: Vec<LibItem>,
                }
                match serde_json::from_str::<Body>(&body) {
                    Result::Ok(body)
                        if body.auth_key == "auth_key"
                            && body.collection == "libraryItem"
                            && body.changes.len() == 3
                            && body.changes.contains(&LOCAL_NEWER_ITEM)
                            && body.changes.contains(&LOCAL_ONLY_ITEM)
                            && body.changes.contains(&LOCAL_WATCHED_ITEM) =>
                    {
                        Box::new(future::ok(Box::new(APIResult::Ok {
                            result: SuccessResponse { success: True {} },
                        }) as Box<dyn Any>))
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
                match serde_json::from_str::<Body>(&body) {
                    Result::Ok(body)
                        if body.auth_key == "auth_key"
                            && body.collection == "libraryItem"
                            && body.all == false
                            && body.ids.len() == 2
                            && body.ids.contains(&REMOTE_ONLY_ITEM.id)
                            && body.ids.contains(&REMOTE_NEWER_ITEM.id) =>
                    {
                        Box::new(future::ok(Box::new(APIResult::Ok {
                            result: vec![REMOTE_ONLY_ITEM.to_owned(), REMOTE_NEWER_ITEM.to_owned()],
                        }) as Box<dyn Any>))
                    }
                    _ => default_fetch_handler(request),
                }
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                time: Env::now(),
                                from: "tests".to_owned(),
                            },
                        },
                    }),
                    ..Default::default()
                },
                library: LibBucket {
                    uid: Some("user_id".to_owned()),
                    items: vec![
                        (LOCAL_ONLY_ITEM.id.to_owned(), LOCAL_ONLY_ITEM.to_owned()),
                        (LOCAL_NEWER_ITEM.id.to_owned(), LOCAL_NEWER_ITEM.to_owned()),
                        (
                            REMOTE_NEWER_ITEM.id.to_owned(),
                            LibItem {
                                mtime: REMOTE_NEWER_ITEM.mtime - Duration::days(1),
                                ..REMOTE_NEWER_ITEM.to_owned()
                            },
                        ),
                        (
                            LOCAL_NOT_WATCHED_ITEM.id.to_owned(),
                            LOCAL_NOT_WATCHED_ITEM.to_owned(),
                        ),
                        (
                            LOCAL_WATCHED_ITEM.id.to_owned(),
                            LOCAL_WATCHED_ITEM.to_owned(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.library,
        LibBucket {
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
                    LOCAL_NOT_WATCHED_ITEM.id.to_owned(),
                    LOCAL_NOT_WATCHED_ITEM.to_owned()
                ),
                (
                    LOCAL_WATCHED_ITEM.id.to_owned(),
                    LOCAL_WATCHED_ITEM.to_owned()
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
            .map(|data| serde_json::from_str::<LibBucket>(&data).unwrap()),
        Some(LibBucket::new(
            Some("user_id".to_owned()),
            vec![
                REMOTE_ONLY_ITEM.to_owned(),
                LOCAL_ONLY_ITEM.to_owned(),
                REMOTE_NEWER_ITEM.to_owned(),
                LOCAL_NEWER_ITEM.to_owned(),
                LOCAL_NOT_WATCHED_ITEM.to_owned(),
                LOCAL_WATCHED_ITEM.to_owned(),
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
        REQUESTS.read().unwrap().get(0).unwrap().url,
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
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match &request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreMeta"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItemModified>::new(),
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                time: Env::now(),
                                from: "tests".to_owned(),
                            },
                        },
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI))));
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().url,
        "https://api.strem.io/api/datastoreMeta".to_owned(),
        "datastoreMeta request has been sent"
    );
}
