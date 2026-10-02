use crate::{
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        EnvError, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::{
        addon::{Descriptor, Manifest, ManifestResource},
        profile::Profile,
        resource::MetaItemPreview,
    },
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS},
};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[derive(Model, Clone, Default)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
}

fn library_addon(transport_url: &str, types: &[&str]) -> Descriptor {
    Descriptor {
        manifest: Manifest {
            resources: vec![ManifestResource::Full {
                name: "library".to_owned(),
                types: Some(types.iter().map(|r#type| r#type.to_string()).collect()),
                id_prefixes: Some(vec!["tt".to_owned()]),
            }],
            ..Default::default()
        },
        transport_url: transport_url.parse().unwrap(),
        flags: Default::default(),
    }
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, .. } if url.starts_with("https://tracker/library/") => {
            future::err(EnvError::Fetch("ignored".to_owned())).boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

fn dispatch(runtime: &Runtime<TestEnv, TestModel>, action: ActionCtx) {
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(action),
        })
    });
}

fn request_urls() -> Vec<String> {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .map(|request| request.url.to_owned())
        .collect()
}

#[test]
fn library_add_and_remove_notify_supporting_addons_once() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![
                        library_addon("https://tracker/manifest.json", &["series"]),
                        library_addon("https://movies-only/manifest.json", &["movie"]),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        vec![],
        1000,
    );
    let meta_preview = MetaItemPreview {
        id: "tt1".to_owned(),
        r#type: "series".to_owned(),
        ..Default::default()
    };

    dispatch(&runtime, ActionCtx::AddToLibrary(meta_preview.to_owned()));
    dispatch(&runtime, ActionCtx::AddToLibrary(meta_preview));
    dispatch(&runtime, ActionCtx::RemoveFromLibrary("tt1".to_owned()));
    dispatch(&runtime, ActionCtx::RemoveFromLibrary("tt1".to_owned()));

    assert_eq!(
        request_urls(),
        vec![
            "https://tracker/library/series/tt1/action=libraryAdd.json",
            "https://tracker/library/series/tt1/action=libraryRemove.json",
        ],
        "only the addon supporting the type is notified, and only on real changes"
    );
}

#[test]
fn library_item_mark_as_watched_notifies_addons() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![library_addon("https://tracker/manifest.json", &["movie"])],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        vec![],
        1000,
    );
    let meta_preview = MetaItemPreview {
        id: "tt2".to_owned(),
        r#type: "movie".to_owned(),
        ..Default::default()
    };

    dispatch(
        &runtime,
        ActionCtx::MetaItemMarkAsWatched {
            meta_item: meta_preview,
            is_watched: true,
        },
    );
    dispatch(
        &runtime,
        ActionCtx::LibraryItemMarkAsWatched {
            id: "tt2".to_owned(),
            is_watched: false,
        },
    );

    assert_eq!(
        request_urls(),
        vec![
            "https://tracker/library/movie/tt2/action=watched.json",
            "https://tracker/library/movie/tt2/action=unwatched.json",
        ],
    );
}
