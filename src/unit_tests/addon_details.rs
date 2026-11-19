use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures::future;
use stremio_derive::Model;

use crate::{
    models::{
        addon_details::{AddonDetails, Selected},
        common::Loadable,
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad},
        EnvError, EnvFutureExt, Runtime, RuntimeAction,
    },
    types::addon::Manifest,
    unit_tests::{default_fetch_handler, TestEnv, FETCH_HANDLER, REQUESTS},
};

#[test]
fn reopening_addon_details_retries_a_failed_manifest() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        addon_details: AddonDetails,
    }

    const MANIFEST_URL: &str = "https://addon.example/manifest.json";
    let _guard = TestEnv::reset().unwrap();
    let attempts = AtomicUsize::new(0);
    let manifest = Manifest {
        id: "example.addon".to_owned(),
        ..Default::default()
    };
    *FETCH_HANDLER.write().unwrap() = Box::new({
        let manifest = manifest.clone();
        move |request| {
            if request.url != MANIFEST_URL {
                return default_fetch_handler(request);
            }
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                future::err(EnvError::Fetch("manifest unavailable".to_owned())).boxed_env()
            } else {
                future::ok(Box::new(manifest.clone()) as Box<dyn Any + Send>).boxed_env()
            }
        }
    });

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel::default(), vec![], 1000);
    let selected = Selected {
        transport_url: MANIFEST_URL.parse().unwrap(),
    };
    let open_addon = || {
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::AddonDetails(selected.clone())),
            });
        });
    };

    open_addon();
    let model = runtime.model().unwrap();
    assert!(matches!(
        model.addon_details.remote_addon.as_ref().unwrap().content,
        Loadable::Err(_)
    ));
    drop(model);

    open_addon();
    let model = runtime.model().unwrap();
    let addon = model
        .addon_details
        .remote_addon
        .as_ref()
        .and_then(|remote| remote.content.ready())
        .expect("reopening should retry the manifest request");
    assert_eq!(addon.manifest, manifest);
    drop(model);

    open_addon();
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| request.url == MANIFEST_URL));
}
