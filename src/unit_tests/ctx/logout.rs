use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, SuccessResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::types::True;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE,
};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_logout() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
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
    let profile = Profile {
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
    };
    let library = LibraryBucket {
        uid: profile.uid(),
        ..Default::default()
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(profile.uid(), vec![])).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(profile.uid(), vec![])).unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                profile,
                library,
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
            action: Action::Ctx(ActionCtx::Logout),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        Default::default(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.model().unwrap().ctx.library,
        Default::default(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(data).unwrap() == Default::default()
            }),
        "profile updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(data).unwrap() == Default::default()
            }),
        "recent library updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(data).unwrap() == Default::default()
            }),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/logout".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}".to_owned(),
            ..Default::default()
        },
        "Logout request has been sent"
    );
}
