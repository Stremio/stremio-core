use crate::models::catalog_with_filters::{CatalogWithFilters, Selected};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::ResourceResponse;
use crate::types::resource::MetaItemPreview;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, EVENTS, FETCH_HANDLER, REQUESTS, STATES,
};
use enclose::enclose;
use futures::future;
use std::any::Any;
use std::sync::{Arc, RwLock};
use stremio_derive::Model;

#[test]
fn catalog_with_filters() {
    #[derive(Model, Default, Clone)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        discover: CatalogWithFilters<MetaItemPreview>,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if url == "https://v3-cinemeta.strem.io/catalog/movie/top.json"
                    && method == "GET" =>
            {
                future::ok(Box::new(ResourceResponse::Metas {
                    metas: vec![MetaItemPreview::default()],
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let _env_mutex = TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let ctx = Ctx::default();
    let (discover, effects) = CatalogWithFilters::<MetaItemPreview>::new(&ctx.profile);
    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel { ctx, discover }, effects, 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    TestEnv::run2(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            let request = runtime
                .model()
                .unwrap()
                .discover
                .selectable
                .catalogs
                .first()
                .unwrap()
                .request
                .to_owned();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::CatalogWithFilters(Selected { request })),
            });
        }),
    );
    let events = EVENTS.read().unwrap();
    let states = STATES.read().unwrap();
    let requests = REQUESTS.read().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(states.len(), 2);
    assert_eq!(requests.len(), 1);
}
