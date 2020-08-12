use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{APIResult, Auth, SuccessResponse, True, User};
use crate::types::profile::Profile;
use crate::types::{LibBucket, LibItem, LibItemModified};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, NOW, REQUESTS};
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
                    result: Vec::<LibItemModified>::new(),
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-01T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"behaviorHints\":{\"defaultVideoId\":null}}]}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":false}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItem>::new(),
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
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
                    uid: Some("id".to_owned()),
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
            body: "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"removed\":false,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-01T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"lastVidReleased\":null,\"noNotif\":false},\"name\":\"name\",\"type\":\"type_name\",\"poster\":null,\"behaviorHints\":{\"defaultVideoId\":null}}]}"
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
                "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":false}"
                    .to_owned(),
            ..Default::default()
        },
        "datastoreGet request has been sent"
    );
}
