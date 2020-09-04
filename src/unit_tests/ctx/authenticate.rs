use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::api::{
    APIResult, AuthRequest, AuthResponse, CollectionResponse, GDPRConsentWithTime,
};
use crate::types::library::{LibBucket, LibItem};
use crate::types::profile::{Auth, GDPRConsent, Profile, User};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use chrono::prelude::{TimeZone, Utc};
use core::pin::Pin;
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_authenticate_login() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/login"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\"}" =>
            {
                Pin::new(Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
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
                                from: "tests".to_owned(),
                            },
                        }
                    },
                }) as Box<dyn Any>)))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Pin::new(Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>)))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                Pin::new(Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItem>::new(),
                }) as Box<dyn Any>)))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::Authenticate(AuthRequest::Login {
            email: "user_email".into(),
            password: "user_password".into(),
        }),
    ))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile,
        Profile {
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
                        from: "tests".to_owned(),
                    },
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library,
        LibBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        Profile {
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
                        from: "tests".to_owned(),
                    },
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibBucket>(
            &STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibBucket>(
            &STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/login".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\"}".to_owned(),
            ..Default::default()
        },
        "Login request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(1).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/addonCollectionGet".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}"
                .to_owned(),
            ..Default::default()
        },
        "AddonCollectionGet request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(2).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/datastoreGet".to_owned(),
            method: "POST".to_owned(),
            body:
                "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}"
                    .to_owned(),
            ..Default::default()
        },
        "DatastoreGet request has been sent"
    );
}

#[test]
fn actionctx_authenticate_register() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/register"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"from\":\"web\",\"time\":\"2020-01-01T00:00:00Z\"}}" =>
            {
                Pin::new( Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
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
                                from: "tests".to_owned(),
                            },
                        }
                    },
                }) as Box<dyn Any>)))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Pin::new(Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>)))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                Pin::new(Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItem>::new(),
                }) as Box<dyn Any>)))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::Authenticate(AuthRequest::Register {
            email: "user_email".into(),
            password: "user_password".into(),
            gdpr_consent: GDPRConsentWithTime {
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: false,
                    from: "web".to_owned(),
                },
                time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
            },
        }),
    ))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile,
        Profile {
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
                        from: "tests".to_owned(),
                    },
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library,
        LibBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        Profile {
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
                        from: "tests".to_owned(),
                    },
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibBucket>(
            &STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibBucket>(
            &STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/register".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"from\":\"web\",\"time\":\"2020-01-01T00:00:00Z\"}}".to_owned(),
            ..Default::default()
        },
        "Register request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(1).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/addonCollectionGet".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}"
                .to_owned(),
            ..Default::default()
        },
        "AddonCollectionGet request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(2).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/datastoreGet".to_owned(),
            method: "POST".to_owned(),
            body:
                "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}"
                    .to_owned(),
            ..Default::default()
        },
        "DatastoreGet request has been sent"
    );
}
