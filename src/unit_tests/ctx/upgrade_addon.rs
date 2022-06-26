use crate::constants::PROFILE_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Effects, Runtime, RuntimeAction};
use crate::types::addon::{Descriptor, Manifest};
use crate::types::profile::Profile;
use crate::unit_tests::{TestEnv, REQUESTS, STORAGE};
use semver::Version;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_installaddon_upgrade() {
    #[derive(Model, Default)]
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
        transport_url: Url::parse("https://transport_url2").unwrap(),
        flags: Default::default(),
    };
    let _env_mutex = TestEnv::reset();
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon1.to_owned()],
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
            action: Action::Ctx(ActionCtx::UpgradeAddon(addon2.to_owned())),
        })
    });

    assert_eq!(
        runtime.model().unwrap().ctx.profile.addons,
        vec![addon2.to_owned()],
        "addon upgrade successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons == vec![addon2.to_owned()]
            }),
        "addon upgrade successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
