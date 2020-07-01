use super::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{
    APIResult, Auth, AuthRequest, AuthResponse, CollectionResponse, GDPRConsent, SuccessResponse,
    True, User,
};
use crate::types::profile::{Profile, UID};
use crate::types::{LibBucket, LibItem};
use chrono::prelude::*;
use futures::future;
use lazy_static::lazy_static;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_logout() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    let profile = Profile {
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
    };
    let library = LibBucket {
        uid: profile.uid(),
        ..Default::default()
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                library,
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Logout))));
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_none(),
        "profile updated successfully in memory"
    );
    assert!(
        runtime.app.read().unwrap().ctx.library.uid.is_none()
            && runtime.app.read().unwrap().ctx.library.items.is_empty(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .auth
                    .is_none()
            }),
        "profile updated successfully in storage"
    );
    // TODO library updated successfully in storage
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been send"
    );
    assert!(
        match REQUESTS.read().unwrap().get(0).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
                true,
            _ => false,
        },
        "Logout request has been send"
    );
}

#[test]
fn actionctx_login() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_none(),
        "there is no user"
    );
    lazy_static! {
        #[derive(Debug)]
        static ref TEST_USER: User = User {
            id: "user_id".to_owned(),
            email: "user_email".to_owned(),
            fb_id: None,
            avatar: None,
            last_modified: Env::now(),
            date_registered: Env::now(),
        };
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/login"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
                        key: "auth_key".to_owned(),
                        user: TEST_USER.to_owned()
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                let empty_library: Vec<LibItem> = Vec::new();
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: empty_library,
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Authenticate(
            AuthRequest::Login {
                email: "user_email".into(),
                password: "user_password".into(),
            },
        )))),
    );
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_some(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime
            .app
            .read()
            .unwrap()
            .ctx
            .profile
            .auth
            .to_owned()
            .unwrap()
            .user,
        *TEST_USER,
        "logged user equal to the test user"
    );
    assert!(
        runtime.app.read().unwrap().ctx.library.uid.is_some()
            && runtime.app.read().unwrap().ctx.library.items.is_empty(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .auth
                    .is_some()
            }),
        "profile updated successfully in storage"
    );
    // TODO library updated successfully in storage
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three request has been send"
    );
    assert!(
        match REQUESTS.read().unwrap().get(0).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/login"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\"}" =>
                true,
            _ => false,
        },
        "Login request has been send"
    );
    assert!(
        match REQUESTS.read().unwrap().get(1).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
                true,
            _ => false,
        },
        "AddonCollectionGet request has been send"
    );
    assert!(
        match REQUESTS.read().unwrap().get(2).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
                true,
            _ => false,
        },
        "DatastoreGet request has been send"
    );
}

#[test]
fn actionctx_signup() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_none(),
        "there is no user"
    );
    lazy_static! {
        #[derive(Debug)]
        static ref TEST_USER: User = User {
            id: "user_id".to_owned(),
            email: "user_email".to_owned(),
            fb_id: None,
            avatar: None,
            last_modified: Env::now(),
            date_registered: Env::now(),
        };
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/register"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"time\":\"2020-01-01T00:00:00Z\",\"from\":\"web\"}}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
                        key: "auth_key".to_owned(),
                        user: TEST_USER.to_owned()
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                let empty_library: Vec<LibItem> = Vec::new();
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: empty_library,
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Authenticate(
            AuthRequest::Register {
                email: "user_email".into(),
                password: "user_password".into(),
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: false,
                    time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                    from: "web".to_owned(),
                },
            },
        )))),
    );
    assert!(
        runtime.app.read().unwrap().ctx.profile.auth.is_some(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime
            .app
            .read()
            .unwrap()
            .ctx
            .profile
            .auth
            .to_owned()
            .unwrap()
            .user,
        *TEST_USER,
        "registered user equal to the test user"
    );
    assert!(
        runtime.app.read().unwrap().ctx.library.uid.is_some()
            && runtime.app.read().unwrap().ctx.library.items.is_empty(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .auth
                    .is_some()
            }),
        "profile updated successfully in storage"
    );
    // TODO library updated successfully in storage
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert!(
        match REQUESTS.read().unwrap().get(0).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/register"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"time\":\"2020-01-01T00:00:00Z\",\"from\":\"web\"}}" =>
                true,
            _ => false,
        },
        "Register request has been sent"
    );
    assert!(
        match REQUESTS.read().unwrap().get(1).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
                true,
            _ => false,
        },
        "AddonCollectionGet request has been sent"
    );
    assert!(
        match REQUESTS.read().unwrap().get(2).unwrap() {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
                true,
            _ => false,
        },
        "DatastoreGet request has been sent"
    );
}
