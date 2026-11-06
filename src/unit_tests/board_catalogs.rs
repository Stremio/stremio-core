use crate::constants::CATALOG_RESOURCE_NAME;
use crate::models::board_catalogs::{BoardCatalogs, Selected};
use crate::models::common::{spotlight_items, CatalogPage};
use crate::models::common::{Loadable, ResourceLoadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx, ActionLoad};
use crate::runtime::{EnvError, EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture};
use crate::types::addon::{ResourcePath, ResourceRequest, ResourceResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{BoardSpotlight, Profile, Settings};
use crate::types::resource::MetaItemPreview;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS};
use assert_matches::assert_matches;
use futures::{channel::mpsc::Receiver, future};
use std::any::Any;
use stremio_derive::Model;
use url::Url;

#[derive(Model, Clone, Default)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    board_catalogs: BoardCatalogs,
}

const CINEMETA_URL: &str = "https://v3-cinemeta.strem.io/catalog/";
const POPULAR_MOVIES_URL: &str = "https://v3-cinemeta.strem.io/catalog/movie/top.json";
const POPULAR_SERIES_URL: &str = "https://v3-cinemeta.strem.io/catalog/series/top.json";
const CHANNELS_URL: &str = "https://v3-channels.strem.io/catalog/channel/top.json";

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, method, .. } if url.starts_with(CINEMETA_URL) && method == "GET" => {
            future::ok(Box::new(ResourceResponse::Metas {
                metas: vec![MetaItemPreview {
                    id: "tt1".to_owned(),
                    background: Some(Url::parse("https://background").unwrap()),
                    ..Default::default()
                }],
            }) as Box<dyn Any + Send>)
            .boxed_env()
        }
        Request { method, .. } if method == "GET" => {
            future::err(EnvError::Fetch("offline".to_owned())).boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

fn cinemeta_request(r#type: &str, id: &str) -> ResourceRequest {
    ResourceRequest::new(
        Url::parse("https://v3-cinemeta.strem.io/manifest.json").unwrap(),
        ResourcePath::without_extra(CATALOG_RESOURCE_NAME, r#type, id),
    )
}

fn requests_to(url: &str) -> usize {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .filter(|request| request.url == url)
        .count()
}

fn runtime_with(
    board_spotlight: BoardSpotlight,
) -> (
    Runtime<TestEnv, TestModel>,
    Receiver<RuntimeEvent<TestEnv, TestModel>>,
) {
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let ctx = Ctx::new(
        Profile {
            settings: Settings {
                board_spotlight,
                ..Settings::default()
            },
            ..Profile::default()
        },
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    Runtime::<TestEnv, _>::new(
        TestModel {
            ctx,
            board_catalogs: BoardCatalogs::default(),
        },
        vec![],
        1000,
    )
}

fn ready_page(items: Vec<MetaItemPreview>) -> CatalogPage<MetaItemPreview> {
    ResourceLoadable {
        request: cinemeta_request("movie", "top"),
        content: Some(Loadable::Ready(items)),
    }
}

fn load(all: bool) -> RuntimeAction<TestEnv, TestModel> {
    RuntimeAction {
        field: None,
        action: Action::Load(ActionLoad::BoardCatalogs(Selected { all })),
    }
}

#[test]
fn loads_only_the_selected_catalogs() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight::default());
    TestEnv::run(|| runtime.dispatch(load(false)));
    let catalogs = &runtime.model().unwrap().board_catalogs.catalogs;
    assert_eq!(catalogs.len(), 6);
    let loaded = catalogs
        .iter()
        .filter(|page| page.content.is_some())
        .map(|page| page.request.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        loaded,
        vec![
            cinemeta_request("movie", "top"),
            cinemeta_request("series", "top")
        ]
    );
    assert_eq!(REQUESTS.read().unwrap().len(), 2);
}

#[test]
fn loads_every_catalog_once() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight::default());
    TestEnv::run(|| runtime.dispatch(load(true)));
    let catalogs = &runtime.model().unwrap().board_catalogs.catalogs;
    assert_eq!(catalogs.len(), 6);
    assert!(catalogs.iter().all(|page| page.content.is_some()));
    assert_eq!(REQUESTS.read().unwrap().len(), 6);
    assert_matches!(
        catalogs.iter().find(|page| page
            .request
            .base
            .as_str()
            .starts_with("https://v3-channels")),
        Some(ResourceLoadable {
            content: Some(Loadable::Err(_)),
            ..
        })
    );
}

#[test]
fn reloads_only_the_failed_catalogs() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight::default());
    TestEnv::run(|| runtime.dispatch(load(true)));
    TestEnv::run(|| runtime.dispatch(load(true)));
    assert_eq!(requests_to(POPULAR_MOVIES_URL), 1);
    assert_eq!(requests_to(CHANNELS_URL), 2);
}

#[test]
fn loads_the_new_pick() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight {
        catalogs: vec![cinemeta_request("movie", "top")],
        ..BoardSpotlight::default()
    });
    TestEnv::run(|| {
        runtime.dispatch(load(false));
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::UpdateSettings(Settings {
                board_spotlight: BoardSpotlight {
                    catalogs: vec![
                        cinemeta_request("movie", "top"),
                        cinemeta_request("series", "top"),
                    ],
                    ..BoardSpotlight::default()
                },
                ..Settings::default()
            })),
        });
    });
    let catalogs = &runtime.model().unwrap().board_catalogs.catalogs;
    assert_matches!(
        catalogs
            .iter()
            .find(|page| page.request == cinemeta_request("series", "top")),
        Some(ResourceLoadable {
            content: Some(Loadable::Ready(_)),
            ..
        })
    );
    assert_eq!(requests_to(POPULAR_MOVIES_URL), 1);
    assert_eq!(requests_to(POPULAR_SERIES_URL), 1);
    assert_eq!(REQUESTS.read().unwrap().len(), 2);
}

#[test]
fn spotlight_items_are_unique_with_background() {
    let background = Some(Url::parse("https://background").unwrap());
    let items = (0..10)
        .map(|index| MetaItemPreview {
            id: format!("tt{}", index / 2),
            background: if index % 3 == 0 {
                None
            } else {
                background.to_owned()
            },
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let page = ready_page(items);
    let items = spotlight_items(std::iter::once(&page));
    let ids = items
        .iter()
        .map(|(_, item)| item.id.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["tt0", "tt1", "tt2", "tt3", "tt4"]);
    assert!(items.iter().all(|(_, item)| item.background.is_some()));
    let many = (0..20)
        .map(|index| MetaItemPreview {
            id: format!("tt{index}"),
            background: background.to_owned(),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let page = ready_page(many);
    assert_eq!(spotlight_items(std::iter::once(&page)).len(), 6);
}
