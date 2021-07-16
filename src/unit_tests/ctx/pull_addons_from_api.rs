use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Effects, Env, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::{Descriptor, Manifest};
use crate::types::api::{APIResult, CollectionResponse};
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE,
};
use futures::{future, FutureExt};
use semver::Version;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_pulladdonsfromapi() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let official_addon = OFFICIAL_ADDONS.first().unwrap();
    TestEnv::reset();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            version: Version::new(0, 0, 1),
                            ..official_addon.manifest.to_owned()
                        },
                        transport_url: Url::parse("https://transport_url").unwrap(),
                        flags: official_addon.flags.to_owned(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::PullAddonsFromAPI),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        vec![official_addon.to_owned()],
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons
                    == vec![official_addon.to_owned()]
            }),
        "addons updated successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_pulladdonsfromapi_with_user() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: OFFICIAL_ADDONS.to_owned(),
                        last_modified: TestEnv::now(),
                    },
                }) as Box<dyn Any>).boxed_local()
            }
            _ => default_fetch_handler(request),
        }
    }
    TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                            },
                        },
                    }),
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            id: "id".to_owned(),
                            version: Version::new(0, 0, 1),
                            name: "name".to_owned(),
                            contact_email: None,
                            description: None,
                            logo: None,
                            background: None,
                            types: vec![],
                            resources: vec![],
                            id_prefixes: None,
                            catalogs: vec![],
                            addon_catalogs: vec![],
                            behavior_hints: Default::default(),
                        },
                        transport_url: Url::parse("https://transport_url").unwrap(),
                        flags: Default::default(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        Effects::none().unchanged(),
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::PullAddonsFromAPI),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        OFFICIAL_ADDONS.to_owned(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons == OFFICIAL_ADDONS.to_owned()
            }),
        "addons updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().url,
        "https://api.strem.io/api/addonCollectionGet".to_owned(),
        "addonCollectionGet request has been sent"
    );
}
