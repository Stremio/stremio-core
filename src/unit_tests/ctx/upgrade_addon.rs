use crate::constants::PROFILE_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Runtime, RuntimeAction};
use crate::types::addon::{Descriptor, Manifest};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{TestEnv, REQUESTS, STORAGE};
use semver::Version;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_addon_upgrade() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let addon1 = Descriptor {
        manifest: Manifest {
            id: "id1".to_owned(),
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
    };
    let addon1_update = Descriptor {
        manifest: Manifest {
            id: "id1".to_owned(),
            version: Version::new(0, 0, 2),
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
    };
    let addon2 = Descriptor {
        manifest: Manifest {
            id: "id2".to_owned(),
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
        transport_url: Url::parse("https://transport_url_other").unwrap(),
        flags: Default::default(),
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    addons: vec![addon1, addon2.to_owned()],
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
            action: Action::Ctx(ActionCtx::UpgradeAddon(addon1_update.to_owned())),
        })
    });
    let expected = vec![addon1_update, addon2];

    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        expected,
        "addon upgrade successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(data).unwrap().addons == expected
            }),
        "addon upgrade successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_addon_upgrade_fail_due_to_different_url() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let addon1 = Descriptor {
        manifest: Manifest {
            id: "id1".to_owned(),
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
        transport_url: Url::parse("https://transport_url1").unwrap(),
        flags: Default::default(),
    };
    let addon2 = Descriptor {
        manifest: Manifest {
            id: "id1".to_owned(),
            version: Version::new(0, 0, 2),
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
        transport_url: Url::parse("https://transport_url2").unwrap(),
        flags: Default::default(),
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    addons: vec![addon1.to_owned()],
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
            action: Action::Ctx(ActionCtx::UpgradeAddon(addon2.to_owned())),
        })
    });

    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        vec![addon1],
        "addon was not updated"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
