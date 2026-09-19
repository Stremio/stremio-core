use std::any::Any;
use std::sync::{Arc, RwLock};

use enclose::enclose;
use futures::future;
use stremio_derive::Model;
use url::Url;

use crate::{
    models::{
        addon_details::{AddonDetails, Selected},
        common::{DescriptorLoadable, Loadable},
        ctx::Ctx,
    },
    runtime::{msg::Action, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture},
    types::addon::Manifest,
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, RESOLVED_URLS, STATES},
};

const SHORTENER: &str = "https://short.url/abc";
const REAL_ADDON: &str = "https://addon.example/manifest.json";

#[derive(Model, Default, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    addon_details: AddonDetails,
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    if request.url == SHORTENER {
        future::ok(Box::new(Manifest::default()) as Box<dyn Any + Send>).boxed_env()
    } else {
        default_fetch_handler(request)
    }
}

/// Dispatch a `Load(AddonDetails)` for the shortener URL, run the runtime and
/// collect the model states via `STATES`.
fn run_with_transport(resolved: Option<&str>) {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    if let Some(resolved) = resolved {
        RESOLVED_URLS
            .write()
            .unwrap()
            .insert(SHORTENER.to_owned(), resolved.to_owned());
    }

    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel::default(), vec![], 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(crate::runtime::msg::ActionLoad::AddonDetails(
                    Selected {
                        transport_url: Url::parse(SHORTENER).unwrap(),
                    },
                )),
            });
        }),
    );
}

/// Returns the `remote_addon` `transport_url` from the last collected state.
fn last_transport_url() -> String {
    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    let last = states.last().expect("expected at least one state");
    match &last.addon_details.remote_addon {
        Some(DescriptorLoadable {
            transport_url,
            content: Loadable::Ready(..),
            ..
        }) => transport_url.to_string(),
        _ => panic!("expected a Ready remote addon"),
    }
}

#[test]
fn resolves_shortener_redirect_and_adopts_transport_url() {
    run_with_transport(Some(REAL_ADDON));
    assert_eq!(last_transport_url(), REAL_ADDON);
}

#[test]
fn without_redirect_keeps_entered_url() {
    run_with_transport(None);
    assert_eq!(last_transport_url(), SHORTENER);
}

#[test]
fn ignores_redirect_with_invalid_suffix() {
    run_with_transport(Some("https://evil.example/not_an_addon"));
    assert_eq!(last_transport_url(), SHORTENER);
}
