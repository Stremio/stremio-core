use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::{Descriptor, Manifest, ResourceResponse};
use crate::types::api::{APIResult, CollectionResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE,
};
use futures::future;
use semver::Version;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_pulladdonsfromapi() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let official_addon = OFFICIAL_ADDONS.first().unwrap();
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
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
            .is_some_and(|data| {
                serde_json::from_str::<Profile>(data).unwrap().addons
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
fn guest_refresh_updates_installed_community_addons() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    const COMMUNITY_CATALOG_URL: &str =
        "https://v3-cinemeta.strem.io/addon_catalog/all/community.json";

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let installed = Descriptor {
        transport_url: "https://addon.example/manifest.json".parse().unwrap(),
        manifest: serde_json::from_value(serde_json::json!({
            "id": "community.addon",
            "name": "Community addon",
            "version": "1.0.0",
            "types": ["movie"],
            "resources": ["catalog"],
        }))
        .unwrap(),
        flags: Default::default(),
    };
    let mut configured = installed.clone();
    configured.transport_url.set_query(Some("config=custom"));
    let mut update = installed.clone();
    update.manifest.version = Version::new(2, 0, 0);
    let mut different_url = update.clone();
    different_url.manifest.version = Version::new(3, 0, 0);
    different_url.transport_url = "https://other.example/manifest.json".parse().unwrap();
    let mut different_id = update.clone();
    different_id.manifest.version = Version::new(3, 0, 0);
    different_id.manifest.id = "other.addon".into();

    *FETCH_HANDLER.write().unwrap() = Box::new({
        let update = update.clone();
        move |request| {
            if request.url == COMMUNITY_CATALOG_URL && request.method == "GET" {
                future::ok(Box::new(ResourceResponse::Addons {
                    addons: vec![different_url.clone(), different_id.clone(), update.clone()],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            } else {
                default_fetch_handler(request)
            }
        }
    });
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![installed, configured.clone()],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::PullAddonsFromAPI),
        });
    });

    let expected = vec![update, configured];
    assert_eq!(runtime.model().unwrap().ctx.profile.addons, expected);
    let storage = STORAGE.read().unwrap();
    let saved_profile: Profile = serde_json::from_str(&storage[PROFILE_STORAGE_KEY]).unwrap();
    assert_eq!(saved_profile.addons, expected);
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, COMMUNITY_CATALOG_URL);
}

#[test]
fn actionctx_pulladdonsfromapi_with_user() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                future::ok(Box::new(APIResult::Ok(CollectionResponse {
                        addons: OFFICIAL_ADDONS.to_owned(),
                        last_modified: TestEnv::now(),
                    },
                )) as Box<dyn Any + Send>).boxed_env()
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
                            id: "user_id".into(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            apple_id: None,
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
                            ..Default::default()
                        },
                    }),
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            id: "id".into(),
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
            .is_some_and(|data| {
                serde_json::from_str::<Profile>(data).unwrap().addons == OFFICIAL_ADDONS.to_owned()
            }),
        "addons updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().url,
        "https://api.strem.io/api/addonCollectionGet".to_owned(),
        "addonCollectionGet request has been sent"
    );
}
