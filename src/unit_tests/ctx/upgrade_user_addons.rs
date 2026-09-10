use std::any::Any;

use futures::future;
use semver::Version;
use stremio_derive::Model;
use url::Url;

use crate::{
    constants::PROFILE_STORAGE_KEY,
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx, Internal, Msg},
        Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture, Update,
    },
    types::{
        addon::{Descriptor, DescriptorFlags, Manifest, ManifestBehaviorHints},
        events::DismissedEventsBucket,
        library::LibraryBucket,
        notifications::NotificationsBucket,
        profile::{Auth, AuthKey, GDPRConsent, Profile, User},
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS, STORAGE},
};

fn manifest(id: &str, version: Version) -> Manifest {
    Manifest {
        id: id.to_owned(),
        version,
        name: id.to_owned(),
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
    }
}

#[test]
fn actionctx_upgradeuseraddons_buckets() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    // addon1: regular, has an upgrade available (0.0.1 → 0.0.2)
    // addon2: regular, already up to date (fetched manifest is same version)
    // addon3: protected, must be skipped without any HTTP request
    // addon4: configuration_required, must be skipped without any HTTP request
    let addon1 = Descriptor {
        manifest: manifest("addon1", Version::new(0, 0, 1)),
        transport_url: Url::parse("https://addon1.example/manifest.json").unwrap(),
        flags: Default::default(),
    };
    let addon2 = Descriptor {
        manifest: manifest("addon2", Version::new(1, 0, 0)),
        transport_url: Url::parse("https://addon2.example/manifest.json").unwrap(),
        flags: Default::default(),
    };
    let addon3 = Descriptor {
        manifest: manifest("addon3", Version::new(0, 5, 0)),
        transport_url: Url::parse("https://addon3.example/manifest.json").unwrap(),
        flags: DescriptorFlags {
            official: false,
            protected: true,
        },
    };
    let addon4 = Descriptor {
        manifest: Manifest {
            behavior_hints: ManifestBehaviorHints {
                configuration_required: true,
                ..Default::default()
            },
            ..manifest("addon4", Version::new(0, 5, 0))
        },
        transport_url: Url::parse("https://addon4.example/manifest.json").unwrap(),
        flags: Default::default(),
    };

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request.url.as_str() {
            "https://addon1.example/manifest.json" => future::ok(Box::new(Manifest {
                id: "addon1".to_owned(),
                version: Version::new(0, 0, 2),
                name: "addon1".to_owned(),
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
            })
                as Box<dyn Any + Send>)
            .boxed_env(),
            "https://addon2.example/manifest.json" => future::ok(Box::new(Manifest {
                id: "addon2".to_owned(),
                version: Version::new(1, 0, 0),
                name: "addon2".to_owned(),
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
            })
                as Box<dyn Any + Send>)
            .boxed_env(),
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    addons: vec![
                        addon1.to_owned(),
                        addon2.to_owned(),
                        addon3.to_owned(),
                        addon4.to_owned(),
                    ],
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
            action: Action::Ctx(ActionCtx::UpgradeUserAddons),
        })
    });

    let addons_after = &runtime.model().unwrap().ctx.profile.addons;
    assert_eq!(addons_after.len(), 4, "addon count is preserved");
    assert_eq!(
        addons_after[0].manifest.version,
        Version::new(0, 0, 2),
        "addon1 was upgraded in place"
    );
    assert_eq!(
        addons_after[0].transport_url, addon1.transport_url,
        "addon1 keeps its transport_url"
    );
    assert_eq!(
        addons_after[1], addon2,
        "addon2 is unchanged (already up to date)"
    );
    assert_eq!(addons_after[2], addon3, "protected addon is untouched");
    assert_eq!(
        addons_after[3], addon4,
        "configuration-required addon is untouched"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .is_some_and(|data| {
                let stored: Profile = serde_json::from_str(data).unwrap();
                stored.addons[0].manifest.version == Version::new(0, 0, 2)
            }),
        "upgraded profile is persisted to storage"
    );
    let requests = REQUESTS.read().unwrap();
    assert_eq!(
        requests.len(),
        2,
        "exactly two manifest fetches were made (addon1 + addon2, not the skipped ones)"
    );
    let urls: Vec<&str> = requests.iter().map(|r| r.url.as_str()).collect();
    assert!(urls.contains(&"https://addon1.example/manifest.json"));
    assert!(urls.contains(&"https://addon2.example/manifest.json"));
    assert!(!urls.contains(&"https://addon3.example/manifest.json"));
    assert!(!urls.contains(&"https://addon4.example/manifest.json"));
}

#[test]
fn actionctx_upgradeuseraddons_locked_short_circuits() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let addon = Descriptor {
        manifest: manifest("addon1", Version::new(0, 0, 1)),
        transport_url: Url::parse("https://addon1.example/manifest.json").unwrap(),
        flags: Default::default(),
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    addons: vec![addon.to_owned()],
                    addons_locked: true,
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
            action: Action::Ctx(ActionCtx::UpgradeUserAddons),
        })
    });
    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        vec![addon],
        "no addons were upgraded when addons are locked"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "no manifest fetches were issued when addons are locked"
    );
}

fn test_user(id: &str) -> User {
    User {
        id: id.into(),
        email: format!("{id}@example.test"),
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
    }
}

// Race regression: a user dispatches `UpgradeUserAddons` under auth A, then logs out / swaps
// to auth B before the concurrent manifest fetches resolve. The result must be discarded —
// otherwise it would mutate (and persist + push) addons on the wrong profile.
#[test]
fn actionctx_upgradeuseraddons_drops_results_after_account_swap() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    let addon_v1 = Descriptor {
        manifest: manifest("addon1", Version::new(0, 0, 1)),
        transport_url: Url::parse("https://addon1.example/manifest.json").unwrap(),
        flags: Default::default(),
    };
    let upgraded_manifest = manifest("addon1", Version::new(0, 0, 2));

    // Profile is now logged in as user_b — but the in-flight result was dispatched under user_a.
    let mut ctx = Ctx::new(
        Profile {
            auth: Some(Auth {
                key: AuthKey("user_b_key".to_owned()),
                user: test_user("user_b"),
            }),
            addons: vec![addon_v1.clone()],
            ..Default::default()
        },
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );

    let stale_msg = Msg::Internal(Internal::UserAddonsManifestsResult {
        auth_key: Some(AuthKey("user_a_key".to_owned())),
        results: vec![(addon_v1.transport_url.clone(), Ok(upgraded_manifest))],
    });
    let _effects = <Ctx as Update<TestEnv>>::update(&mut ctx, &stale_msg);

    assert_eq!(
        ctx.profile.addons,
        vec![addon_v1],
        "stale UserAddonsManifestsResult must not mutate the current profile's addons"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "stale UserAddonsManifestsResult must not push the mutated collection to the API"
    );
}

// Race regression: same user, but `addons_locked` flipped to true between dispatch and result
// (e.g. an `AddonsAPIResult` error arrived in the meantime). The result must be discarded so
// the lock is honored.
#[test]
fn actionctx_upgradeuseraddons_drops_results_when_locked_mid_flight() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    let addon_v1 = Descriptor {
        manifest: manifest("addon1", Version::new(0, 0, 1)),
        transport_url: Url::parse("https://addon1.example/manifest.json").unwrap(),
        flags: Default::default(),
    };
    let upgraded_manifest = manifest("addon1", Version::new(0, 0, 2));

    let mut ctx = Ctx::new(
        Profile {
            auth: Some(Auth {
                key: AuthKey("user_a_key".to_owned()),
                user: test_user("user_a"),
            }),
            addons: vec![addon_v1.clone()],
            addons_locked: true,
            ..Default::default()
        },
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );

    let in_flight_msg = Msg::Internal(Internal::UserAddonsManifestsResult {
        auth_key: Some(AuthKey("user_a_key".to_owned())),
        results: vec![(addon_v1.transport_url.clone(), Ok(upgraded_manifest))],
    });
    let _effects = <Ctx as Update<TestEnv>>::update(&mut ctx, &in_flight_msg);

    assert_eq!(
        ctx.profile.addons,
        vec![addon_v1],
        "UserAddonsManifestsResult arriving while addons_locked must not mutate addons"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "UserAddonsManifestsResult arriving while addons_locked must not push to the API"
    );
}
