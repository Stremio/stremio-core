use crate::models::catalog_with_filters::{CatalogWithFilters, Selected};
use crate::models::common::{Loadable, ResourceLoadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture};
use crate::types::addon::{ExtraValue, ResourcePath, ResourceRequest, ResourceResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, EVENTS, FETCH_HANDLER, REQUESTS, STATES,
};
use assert_matches::assert_matches;
use enclose::enclose;
use futures::future;
use std::any::Any;
use std::sync::{Arc, RwLock};
use stremio_derive::Model;
use url::Url;

#[test]
fn default_catalog() {
    #[derive(Model, Clone, Debug)]
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
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (discover, effects) = CatalogWithFilters::<MetaItemPreview>::new(&ctx.profile);
    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel { ctx, discover },
        effects.into_iter().collect::<Vec<_>>(),
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::CatalogWithFilters(None)),
            });
        }),
    );
    let events = EVENTS.read().unwrap();
    assert_eq!(events.len(), 2);
    assert_matches!(
        events[0]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Discover
    );
    assert_matches!(
        events[1]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Discover
    );
    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 3);
    assert!(states[1].discover.selectable.next_page.is_none());
    assert_matches!(&states[1].discover.selected, Some(Selected { request }) if request == &states[0].discover.selectable.types.first().unwrap().request);
    assert_matches!(
        states[1].discover.catalog.first(),
        Some(ResourceLoadable {
            content: Some(Loadable::Loading),
            ..
        })
    );
    assert!(states[2].discover.selectable.next_page.is_some());
    assert_matches!(
        states[2].discover.catalog.first(),
        Some(ResourceLoadable {
            content: Some(Loadable::Ready(..)),
            ..
        })
    );
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0],
        Request {
            url: "https://v3-cinemeta.strem.io/catalog/movie/top.json".to_owned(),
            method: "GET".to_owned(),
            headers: Default::default(),
            body: "null".to_owned()
        }
    )
}

#[test]
fn search_catalog() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        discover: CatalogWithFilters<MetaItemPreview>,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, method, .. }
                if url
                    == "https://v3-cinemeta.strem.io/catalog/movie/top/search=Harry%20Potter.json"
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
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (discover, effects) = CatalogWithFilters::<MetaItemPreview>::new(&ctx.profile);
    let (runtime, rx) = Runtime::<TestEnv, _>::new(
        TestModel { ctx, discover },
        effects.into_iter().collect::<Vec<_>>(),
        1000,
    );
    let runtime = Arc::new(RwLock::new(runtime));
    let selected = Selected {
        request: ResourceRequest {
            base: Url::parse("https://v3-cinemeta.strem.io/manifest.json").unwrap(),
            path: ResourcePath {
                resource: "catalog".to_owned(),
                id: "top".to_owned(),
                r#type: "movie".to_owned(),
                extra: vec![ExtraValue {
                    name: "search".to_owned(),
                    value: "Harry Potter".to_owned(),
                }],
            },
        },
    };
    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime, selected => selected) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::CatalogWithFilters(Some(selected))),
            });
        }),
    );
    let events = EVENTS.read().unwrap();
    assert_eq!(events.len(), 2);
    assert_matches!(
        events[0]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Discover
    );
    assert_matches!(
        events[1]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::Discover
    );
    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 3);
    assert!(states[1].discover.selectable.next_page.is_none());
    assert_eq!(states[1].discover.selected, Some(selected));
    assert_matches!(
        states[1].discover.catalog.first(),
        Some(ResourceLoadable {
            content: Some(Loadable::Loading),
            ..
        })
    );
    assert!(states[2].discover.selectable.next_page.is_some());
    assert_matches!(
        states[2].discover.catalog.first(),
        Some(ResourceLoadable {
            content: Some(Loadable::Ready(..)),
            ..
        })
    );
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0],
        Request {
            url: "https://v3-cinemeta.strem.io/catalog/movie/top/search=Harry%20Potter.json"
                .to_owned(),
            method: "GET".to_owned(),
            headers: Default::default(),
            body: "null".to_owned()
        }
    )
}
