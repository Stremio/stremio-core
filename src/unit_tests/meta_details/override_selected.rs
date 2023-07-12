use crate::constants::{CINEMETA_URL, META_RESOURCE_NAME, OFFICIAL_ADDONS, STREAM_RESOURCE_NAME};
use crate::models::ctx::Ctx;
use crate::models::meta_details::{MetaDetails, Selected};
use crate::runtime::msg::{Action, ActionLoad};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::{Descriptor, Manifest, ManifestResource, ResourcePath, ResourceResponse};
use crate::types::profile::Profile;
use crate::types::resource::{MetaItem, MetaItemBehaviorHints, MetaItemPreview, Video};
use crate::unit_tests::test_env::TEST_ENV;
use crate::unit_tests::{
    default_fetch_handler, is_send, is_sync, test_env::TestEnv, Request, FETCH_HANDLER, STATES,
};
use assert_matches::assert_matches;
use enclose::enclose;
use futures::future;
use std::any::Any;
use std::sync::{Arc, RwLock};
use stremio_derive::Model;
use url::Url;

#[tokio::test]
async fn override_selected_default_video_id() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        meta_details: MetaDetails,
    }

    assert!(is_send::<TestModel>());
    assert!(is_sync::<TestModel>());

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, .. } if url == "https://transport_url/meta/movie/tt1.json" => {
                future::ok(Box::new(ResourceResponse::Meta {
                    meta: MetaItem {
                        preview: MetaItemPreview {
                            id: "tt1".to_owned(),
                            r#type: "movie".to_owned(),
                            behavior_hints: MetaItemBehaviorHints {
                                default_video_id: Some("_tt1".to_owned()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        videos: vec![],
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. } if url == "https://transport_url/stream/movie/_tt1.json" => {
                future::ok(
                    Box::new(ResourceResponse::Streams { streams: vec![] }) as Box<dyn Any + Send>
                )
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let env_guard = TEST_ENV.reset();
    TEST_ENV.set_fetch_handler(Box::new(fetch_handler));
    // *test_env.fetch_handler.write().unwrap() = Box::new(fetch_handler);

    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            types: vec!["movie".to_owned()],
                            resources: vec![
                                ManifestResource::Short(META_RESOURCE_NAME.to_owned()),
                                ManifestResource::Short(STREAM_RESOURCE_NAME.to_owned()),
                            ],
                            ..Default::default()
                        },
                        transport_url: Url::parse("https://transport_url/manifest.json").unwrap(),
                        flags: Default::default(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    let env_guard = TEST_ENV
        .run_with_runtime_tokio(
            rx,
            runtime.clone(),
            enclose!((runtime) move || {
                let runtime = runtime.read().unwrap();
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Load(ActionLoad::MetaDetails(Selected {
                        meta_path: ResourcePath {
                            resource: META_RESOURCE_NAME.to_owned(),
                            r#type: "movie".to_owned(),
                            id: "tt1".to_owned(),
                            extra: vec![]
                        },
                        stream_path: None
                    })),
                });
            }),
        )
        .await;

    let states = TEST_ENV.inner.states.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(states.len(), 3);
    assert_matches!(states[0].meta_details.selected, None);
    assert_matches!(
        states[1].meta_details.selected,
        Some(Selected {
            stream_path: None,
            ..
        })
    );
    assert_matches!(
        &states[2].meta_details.selected,
        Some(Selected {
            stream_path: Some(ResourcePath {
                resource,
                r#type,
                id,
                ..
            }),
            ..
        }) if id == "_tt1" && r#type == "movie" && resource == STREAM_RESOURCE_NAME
    );
}

#[tokio::test]
async fn override_selected_only_video_id() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        meta_details: MetaDetails,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, .. } if url == "https://v3-cinemeta.strem.io/meta/movie/tt1.json" => {
                future::ok(Box::new(ResourceResponse::Meta {
                    meta: MetaItem {
                        preview: MetaItemPreview {
                            id: "tt1".to_owned(),
                            r#type: "movie".to_owned(),
                            ..Default::default()
                        },
                        videos: vec![Video {
                            id: "_tt1".to_owned(),
                            ..Default::default()
                        }],
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let env_mutex = TEST_ENV.reset().expect("Should reset/acquire");
    TEST_ENV.set_fetch_handler(Box::new(fetch_handler));

    // *TEST_ENV.fetch_handler.write().unwrap() = Box::new(fetch_handler);
    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: OFFICIAL_ADDONS
                        .iter()
                        .filter(|addon| addon.transport_url == *CINEMETA_URL)
                        .cloned()
                        .collect(),
                    ..Default::default()
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    TEST_ENV
        .run_with_runtime_tokio(
            rx,
            runtime.clone(),
            enclose!((runtime) move || {
                let runtime = runtime.read().unwrap();
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Load(ActionLoad::MetaDetails(Selected {
                        meta_path: ResourcePath {
                            resource: META_RESOURCE_NAME.to_owned(),
                            r#type: "movie".to_owned(),
                            id: "tt1".to_owned(),
                            extra: vec![]
                        },
                        stream_path: None
                    })),
                });
            }),
        )
        .await;
    let states = TEST_ENV.inner.states.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 4);
}

#[tokio::test]
async fn override_selected_meta_id() {
    #[derive(Model, Default, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        meta_details: MetaDetails,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, .. } if url == "https://v3-cinemeta.strem.io/meta/movie/tt1.json" => {
                future::ok(Box::new(ResourceResponse::Meta {
                    meta: MetaItem {
                        preview: MetaItemPreview {
                            id: "tt1".to_owned(),
                            r#type: "movie".to_owned(),
                            ..Default::default()
                        },
                        videos: vec![],
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TEST_ENV.reset().expect("Should lock/acquire");
    TEST_ENV.set_fetch_handler(Box::new(fetch_handler));
    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: OFFICIAL_ADDONS
                        .iter()
                        .filter(|addon| addon.transport_url == *CINEMETA_URL)
                        .cloned()
                        .collect(),
                    ..Default::default()
                },
                ..Default::default()
            },
            meta_details: Default::default(),
        },
        vec![],
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    TEST_ENV
        .run_with_runtime_tokio(
            rx,
            runtime.clone(),
            enclose!((runtime) move || {
                let runtime = runtime.read().unwrap();
                runtime.dispatch(RuntimeAction {
                    field: None,
                    action: Action::Load(ActionLoad::MetaDetails(Selected {
                        meta_path: ResourcePath {
                            resource: META_RESOURCE_NAME.to_owned(),
                            r#type: "movie".to_owned(),
                            id: "tt1".to_owned(),
                            extra: vec![]
                        },
                        stream_path: None
                    })),
                });
            }),
        )
        .await;
    let states = TEST_ENV.inner.states.read().unwrap();
    println!("states: {states:#?}");
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 4);
    assert_matches!(states[0].meta_details.selected, None);
    assert_matches!(
        states[1].meta_details.selected,
        Some(Selected {
            stream_path: None,
            ..
        })
    );
    assert_matches!(
        &states[2].meta_details.selected,
        Some(Selected {
            stream_path: Some(ResourcePath {
                resource,
                r#type,
                id,
                ..
            }),
            ..
        }) if id == "tt1" && r#type == "movie" && resource == STREAM_RESOURCE_NAME
    );
}
