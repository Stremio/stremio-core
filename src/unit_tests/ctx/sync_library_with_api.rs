use crate::constants::LIBRARY_RECENT_STORAGE_KEY;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, Auth, SuccessResponse, True, User};
use crate::types::profile::{Profile, UID};
use crate::types::{LibBucket, LibItem, LibItemModified};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use chrono::prelude::TimeZone;
use chrono::Utc;
use futures::future;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionload_synclibrarywithapi() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    Env::reset();
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                library: LibBucket {
                    uid: None,
                    items: vec![(
                        "id".to_owned(),
                        LibItem {
                            id: "id".to_owned(),
                            type_name: "type_name".to_owned(),
                            name: "name".to_owned(),
                            poster: None,
                            poster_shape: Default::default(),
                            removed: false,
                            temp: false,
                            ctime: Some(Env::now()),
                            mtime: Env::now(),
                            state: Default::default(),
                            behavior_hints: Default::default(),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::SyncLibraryWithAPI))));
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionload_synclibrarywithapi_with_user() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreMeta"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: vec![
                        LibItemModified("id1".to_owned(), Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
                        LibItemModified("id2".to_owned(), Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0))
                    ],
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id3\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-03T00:00:00Z\",\"_mtime\":\"2020-01-03T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[\"id1\",\"id2\"],\"all\":false}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: vec![LibItem {
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
                    }, LibItem {
                        id: "id2".to_owned(),
                        type_name: "type_name".to_owned(),
                        name: "name".to_owned(),
                        poster: None,
                        poster_shape: Default::default(),
                        removed: false,
                        temp: false,
                        ctime: Some(Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0)),
                        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
                        state: Default::default(),
                        behavior_hints: Default::default(),
                    }],
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    let lib_item1 = LibItem {
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
    let lib_item2 = LibItem {
        id: "id2".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 2).and_hms_milli(0, 0, 0, 0),
        state: Default::default(),
        behavior_hints: Default::default(),
    };
    let lib_item3 = LibItem {
        id: "id3".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        removed: false,
        temp: false,
        ctime: Some(Utc.ymd(2020, 1, 3).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 3).and_hms_milli(0, 0, 0, 0),
        state: Default::default(),
        behavior_hints: Default::default(),
    };
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
                        },
                    }),
                    ..Default::default()
                },
                library: LibBucket {
                    uid: Some("user_id".to_owned()),
                    items: vec![("id3".to_owned(), lib_item3.to_owned())]
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
                ("id1".to_owned(), lib_item1.to_owned()),
                ("id2".to_owned(), lib_item2.to_owned()),
                ("id3".to_owned(), lib_item3.to_owned())
            ]
            .into_iter()
            .collect(),
        },
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap()
                    == (
                        Some("user_id".to_owned()),
                        vec![
                            lib_item1.to_owned(),
                            lib_item2.to_owned(),
                            lib_item3.to_owned(),
                        ],
                    )
            }),
        "Library recent slot updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/datastoreMeta".to_owned(),
            method: "POST".to_owned(),
            body: "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\"}".to_owned(),
            ..Default::default()
        },
        "datastoreMeta request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(1).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/datastorePut".to_owned(),
            method: "POST".to_owned(),
            body: "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id3\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-03T00:00:00Z\",\"_mtime\":\"2020-01-03T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"behaviorHints\":{\"defaultVideoId\":null}}]}"
                .to_owned(),
            ..Default::default()
        },
        "datastorePut request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(2).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/datastoreGet".to_owned(),
            method: "POST".to_owned(),
            body:
            "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[\"id1\",\"id2\"],\"all\":false}"
                    .to_owned(),
            ..Default::default()
        },
        "datastoreGet request has been sent"
    );
}
