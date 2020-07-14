use super::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::addons::{Descriptor, DescriptorFlags, Manifest};
use crate::types::api::{
    APIResult, Auth, AuthRequest, AuthResponse, CollectionResponse, GDPRConsent, SuccessResponse,
    True, User,
};
use crate::types::profile::{Profile, UID};
use crate::types::{LibBucket, LibItem};
use chrono::prelude::{TimeZone, Utc};
use futures::future;
use semver::Version;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_logout() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/logout"
                && method == "POST"
                && body == "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
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
            },
        }),
        ..Default::default()
    };
    let library = LibBucket {
        uid: profile.uid(),
        ..Default::default()
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    STORAGE.write().unwrap().insert(
        LIBRARY_STORAGE_KEY.to_owned(),
        serde_json::to_string::<(UID, Vec<LibItem>)>(&(profile.uid(), vec![])).unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                library,
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Logout))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile,
        Default::default(),
        "profile updated successfully in memory"
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.library,
        Default::default(),
        "library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<Profile>(&data).unwrap() == Default::default()
            }),
        "profile updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap() == Default::default()
            }),
        "recent library updated successfully in storage"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_STORAGE_KEY)
            .map_or(true, |data| {
                serde_json::from_str::<(UID, Vec<LibItem>)>(&data).unwrap() == Default::default()
            }),
        "library updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://api.strem.io/api/logout".to_owned(),
            method: "POST".to_owned(),
            body: "{\"type\":\"Logout\",\"authKey\":\"auth_key\"}".to_owned(),
            ..Default::default()
        },
        "Logout request has been sent"
    );
}

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
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                        }
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItem>::new(),
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Authenticate(
            AuthRequest::Login {
                email: "user_email".into(),
                password: "user_password".into(),
            },
        )))),
    );
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
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<(UID, Vec<LibItem>)>(
            &STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        (Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<(UID, Vec<LibItem>)>(
            &STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        (Some("user_id".to_owned()), vec![]),
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
                && body == "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"time\":\"2020-01-01T00:00:00Z\",\"from\":\"web\"}}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: AuthResponse {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                        }
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: vec![],
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>))
            }
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/datastoreGet"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"ids\":[],\"all\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: Vec::<LibItem>::new(),
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::Authenticate(
            AuthRequest::Register {
                email: "user_email".into(),
                password: "user_password".into(),
                gdpr_consent: GDPRConsent {
                    tos: true,
                    privacy: true,
                    marketing: false,
                    time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                    from: "web".to_owned(),
                },
            },
        )))),
    );
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
                },
            }),
            addons: vec![],
            ..Default::default()
        },
        "profile updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<(UID, Vec<LibItem>)>(
            &STORAGE
                .read()
                .unwrap()
                .get(LIBRARY_RECENT_STORAGE_KEY)
                .unwrap()
        )
        .unwrap(),
        (Some("user_id".to_owned()), vec![]),
        "recent library updated successfully in storage"
    );
    assert_eq!(
        serde_json::from_str::<(UID, Vec<LibItem>)>(
            &STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).unwrap()
        )
        .unwrap(),
        (Some("user_id".to_owned()), vec![]),
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
            body: "{\"type\":\"Auth\",\"type\":\"Register\",\"email\":\"user_email\",\"password\":\"user_password\",\"gdpr_consent\":{\"tos\":true,\"privacy\":true,\"marketing\":false,\"time\":\"2020-01-01T00:00:00Z\",\"from\":\"web\"}}".to_owned(),
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

#[test]
fn actionctx_installaddon_install() {
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
        transport_url: "transport_url".to_owned(),
        flags: Default::default(),
    };
    Env::reset();
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::InstallAddon(
            addon.to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons[0],
        addon,
        "addon installed successfully"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons[0],
        addon,
        "addon updated successfully in storage"
    );
}

#[test]
fn actionctx_installaddon_update() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let addon = Descriptor {
        manifest: Manifest {
            id: "id1.0".to_owned(),
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
        transport_url: "transport_url1".to_owned(),
        flags: Default::default(),
    };
    Env::reset();
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![
                        Descriptor {
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
                            transport_url: "transport_url1".to_owned(),
                            flags: Default::default(),
                        },
                        Descriptor {
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
                            transport_url: "transport_url2".to_owned(),
                            flags: Default::default(),
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::InstallAddon(
            addon.to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons.len(),
        2,
        "There are two addons in memory"
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons[0],
        addon,
        "addon updated successfully in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons
            .len(),
        2,
        "There are two addons in storage"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons[0],
        addon,
        "addon updated successfully in storage"
    );
}

#[test]
fn actionctx_installaddon_update_fail() {
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
        transport_url: "transport_url".to_owned(),
        flags: Default::default(),
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&Profile {
            addons: vec![addon.to_owned()],
            ..Default::default()
        })
        .unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon.to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::InstallAddon(
            addon.to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons.len(),
        1,
        "There is one addon in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons
            .len(),
        1,
        "There is one addon in storage"
    );
}

#[test]
fn actionctx_uninstalladdon_uninstall() {
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
        transport_url: "transport_url".to_owned(),
        flags: Default::default(),
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&Profile {
            addons: vec![addon.to_owned()],
            ..Default::default()
        })
        .unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon.to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
            "transport_url".to_owned(),
        )))),
    );
    assert!(
        runtime.app.read().unwrap().ctx.profile.addons.is_empty(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
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
fn actionctx_uninstalladdon_uninstall_with_user() {
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
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: SuccessResponse { success: True {} },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
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
        transport_url: "transport_url".to_owned(),
        flags: Default::default(),
    };
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
            },
        }),
        addons: vec![addon.to_owned()],
        ..Default::default()
    };
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&profile).unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile,
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
            "transport_url".to_owned(),
        )))),
    );
    assert!(
        runtime.app.read().unwrap().ctx.profile.addons.is_empty(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(true, |data| {
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
fn actionctx_uninstalladdon_uninstall_protected() {
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
        transport_url: "transport_url".to_owned(),
        flags: DescriptorFlags {
            official: false,
            protected: true,
            extra: Default::default(),
        },
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&Profile {
            addons: vec![addon.to_owned()],
            ..Default::default()
        })
        .unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon.to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
            "transport_url".to_owned(),
        )))),
    );
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons,
        vec![addon.to_owned()],
        "protected addon is in memory"
    );
    assert_eq!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons,
        vec![addon.to_owned()],
        "protected addon is in storage"
    );
}

#[test]
fn actionctx_uninstalladdon_fail() {
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
        transport_url: "transport_url".to_owned(),
        flags: Default::default(),
    };
    Env::reset();
    STORAGE.write().unwrap().insert(
        PROFILE_STORAGE_KEY.to_owned(),
        serde_json::to_string(&Profile {
            addons: vec![addon.to_owned()],
            ..Default::default()
        })
        .unwrap(),
    );
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![addon.to_owned()],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(
        runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::UninstallAddon(
            "transport_url2".to_owned(),
        )))),
    );
    assert!(
        runtime
            .app
            .read()
            .unwrap()
            .ctx
            .profile
            .addons
            .contains(&addon),
        "addon is in memory"
    );
    assert!(
        serde_json::from_str::<Profile>(&STORAGE.read().unwrap().get(PROFILE_STORAGE_KEY).unwrap())
            .unwrap()
            .addons
            .contains(&addon),
        "addon is in storage"
    );
}
