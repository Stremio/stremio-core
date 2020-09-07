use crate::constants::PROFILE_STORAGE_KEY;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::addon::{Descriptor, DescriptorFlags, Manifest};
use crate::types::api::{APIResult, SuccessResponse, True};
use crate::types::profile::{Auth, GDPRConsent, Profile, User};
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use futures::future;
use semver::Version;
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[test]
fn actionctx_uninstalladdon() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let profile = Profile {
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
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::UninstallAddon(Url::parse("https://transport_url").unwrap()),
    ))));
    assert!(
        runtime.app().unwrap().ctx.profile.addons.is_empty(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .addons
                    .is_empty()
            }),
        "addons updated successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_uninstalladdon_with_user() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionSet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionSet\",\"authKey\":\"auth_key\",\"addons\":[]}" =>
            {
                Box::pin(future::ok(Box::new(APIResult::Ok {
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
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: true,
                    from: "tests".to_owned(),
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
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::UninstallAddon(Url::parse("https://transport_url").unwrap()),
    ))));
    assert!(
        runtime.app().unwrap().ctx.profile.addons.is_empty(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data)
                    .unwrap()
                    .addons
                    .is_empty()
            }),
        "addons updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/addonCollectionSet".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"AddonCollectionSet\",\"authKey\":\"auth_key\",\"addons\":[]}"
                .to_owned(),
            ..Default::default()
        },
        "addonCollectionSet request has been sent"
    );
}

#[test]
fn actionctx_uninstalladdon_protected() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let addon = Descriptor {
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
        flags: DescriptorFlags {
            official: false,
            protected: true,
        },
    };
    let profile = Profile {
        addons: vec![addon.to_owned()],
        ..Default::default()
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::UninstallAddon(Url::parse("https://transport_url").unwrap()),
    ))));
    assert_eq!(
        runtime.app().unwrap().ctx.profile.addons,
        vec![addon.to_owned()],
        "protected addon is in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons == vec![addon.to_owned()]
            }),
        "protected addon is in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_uninstalladdon_not_installed() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let addon = Descriptor {
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
    };
    let profile = Profile {
        addons: vec![addon.to_owned()],
        ..Default::default()
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _rx) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    tokio_current_thread::block_on_all(runtime.dispatch(&Msg::Action(Action::Ctx(
        ActionCtx::UninstallAddon(Url::parse("https://transport_url2").unwrap()),
    ))));
    assert_eq!(
        runtime.app().unwrap().ctx.profile.addons,
        vec![addon.to_owned()],
        "addons in memory not updated"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons == vec![addon.to_owned()]
            }),
        "addons in storage not updated"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
