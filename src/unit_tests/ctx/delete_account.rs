use crate::{
    constants::PROFILE_STORAGE_KEY,
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        api::{APIResult, SuccessResponse},
        events::DismissedEventsBucket,
        library::LibraryBucket,
        notifications::NotificationsBucket,
        profile::{Auth, AuthKey, GDPRConsent, Password, Profile, User},
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
        True,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE},
};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_delete_account() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/deleteUser"
                && method == "POST"
                && body == "{\"type\":\"DeleteAccount\",\"authKey\":\"auth_key\",\"password\":\"password\"}" =>
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

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                profile,
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
            action: Action::Ctx(ActionCtx::DeleteAccount(Password("password".to_owned()))),
        })
    });

    assert_eq!(
        runtime.model().unwrap().ctx.profile,
        Default::default(),
        "profile updated successfully in memory"
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

    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request have been sent"
    );

    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/deleteUser".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"DeleteAccount\",\"authKey\":\"auth_key\",\"password\":\"password\"}"
                .to_owned(),
            ..Default::default()
        },
        "Delete account request has been sent"
    );
}
