use crate::types::events::DismissedEventsBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::{
    constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY},
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        api::{APIResult, AuthRequest, AuthResponse, CollectionResponse, LibraryItemsResponse},
        library::LibraryBucket,
        profile::{Auth, AuthKey, GDPRConsent, Profile, User},
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE},
};

use futures::future;
use std::any::Any;
use stremio_derive::Model;

fn user_fixture() -> User {
    User {
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
    }
}

fn auth_response_fixture() -> AuthResponse {
    AuthResponse {
        key: AuthKey("auth_key".to_owned()),
        user: user_fixture(),
    }
}

fn profile_fixture() -> Profile {
    Profile {
        auth: Some(Auth {
            key: AuthKey("auth_key".to_owned()),
            user: user_fixture(),
        }),
        addons: vec![],
        ..Default::default()
    }
}

#[test]
fn actionctx_authenticate_login() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/login"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\",\"facebook\":false}" =>
            {
                future::ok(Box::new(APIResult::Ok(auth_response_fixture())) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    CollectionResponse {
                        addons: vec![],
                        last_modified: TestEnv::now(),
                    },
                )) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    LibraryItemsResponse::new(),
                )) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
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
            action: Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
                email: "user_email".into(),
                password: "user_password".into(),
                facebook: false,
            })),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        profile_fixture(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        LibraryBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        profile_fixture(),
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/login".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"Login\",\"email\":\"user_email\",\"password\":\"user_password\",\"facebook\":false}".to_owned(),
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
fn actionctx_authenticate_login_with_token() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/loginWithToken"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"LoginWithToken\",\"token\":\"auth_key\"}" =>
            {
                future::ok(Box::new(APIResult::Ok(auth_response_fixture())) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    CollectionResponse {
                        addons: vec![],
                        last_modified: TestEnv::now(),
                    },)
                ) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(LibraryItemsResponse::new(),)) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
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
            action: Action::Ctx(ActionCtx::Authenticate(AuthRequest::LoginWithToken {
                token: "auth_key".into(),
            })),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        profile_fixture(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        LibraryBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        profile_fixture(),
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/loginWithToken".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"LoginWithToken\",\"token\":\"auth_key\"}"
                .to_owned(),
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
fn actionctx_authenticate_facebook() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/authWithFacebook"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Facebook\",\"token\":\"access_token\"}" =>
            {
                future::ok(Box::new(APIResult::Ok(auth_response_fixture())) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    CollectionResponse {
                        addons: vec![],
                        last_modified: TestEnv::now(),
                    },)
                ) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(LibraryItemsResponse::new(),)) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
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
            action: Action::Ctx(ActionCtx::Authenticate(AuthRequest::Facebook {
                token: "access_token".into(),
            })),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        profile_fixture(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        LibraryBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        profile_fixture(),
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/authWithFacebook".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"Facebook\",\"token\":\"access_token\"}".to_owned(),
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
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/register"
                && method == "POST"
                && body == "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"from\":\"tests\"}}" =>
            {
                future::ok(Box::new(APIResult::Ok(auth_response_fixture())) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    CollectionResponse {
                        addons: vec![],
                        last_modified: TestEnv::now(),
                    },
                )) as Box<dyn Any + Send>).boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(LibraryItemsResponse::new(),
                )) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
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
            action: Action::Ctx(ActionCtx::Authenticate(AuthRequest::Register {
                email: "user_email".into(),
                password: "user_password".into(),
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: false,
                    from: Some("tests".to_owned()),
                },
            })),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        profile_fixture(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        LibraryBucket {
            uid: Some("user_id".to_string()),
            ..Default::default()
        },
        "library updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap(),
        profile_fixture(),
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<LibraryBucket>(
            STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        LibraryBucket::new(Some("user_id".to_owned()), vec![]),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        3,
        "Three requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/register".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"from\":\"tests\"}}".to_owned(),
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
