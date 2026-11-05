use crate::constants::CATALOG_RESOURCE_NAME;
use crate::models::catalogs_with_extra::{CatalogsWithExtra, Selected};
use crate::models::common::{Loadable, ResourceLoadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture};
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
    board: CatalogsWithExtra,
}

const POPULAR_MOVIES_URL: &str = "https://v3-cinemeta.strem.io/catalog/movie/top.json";

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, method, .. } if url == POPULAR_MOVIES_URL && method == "GET" => {
            future::ok(Box::new(ResourceResponse::Metas {
                metas: vec![
                    MetaItemPreview::default(),
                    MetaItemPreview {
                        id: "tt1".to_owned(),
                        background: Some(Url::parse("https://background").unwrap()),
                        ..Default::default()
                    },
                ],
            }) as Box<dyn Any + Send>)
            .boxed_env()
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
            board: CatalogsWithExtra::default(),
        },
        vec![],
        1000,
    )
}

fn load_board() -> RuntimeAction<TestEnv, TestModel> {
    RuntimeAction {
        field: None,
        action: Action::Load(ActionLoad::CatalogsWithExtra(Selected {
            r#type: None,
            extra: vec![],
        })),
    }
}

#[test]
fn spotlight_shares_the_catalog_request() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight::default());
    TestEnv::run(|| {
        runtime.dispatch(load_board());
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadRange(0..0)),
        });
    });
    let board = &runtime.model().unwrap().board;
    assert_matches!(
        &board.spotlight,
        Some(ResourceLoadable {
            request,
            content: Some(Loadable::Ready(items)),
        }) if *request == cinemeta_request("movie", "top") && items.len() == 2
    );
    assert_matches!(
        board.catalogs.first().and_then(|catalog| catalog.first()),
        Some(ResourceLoadable {
            request,
            content: Some(Loadable::Ready(items)),
        }) if *request == cinemeta_request("movie", "top") && items.len() == 2
    );
    assert_eq!(
        REQUESTS
            .read()
            .unwrap()
            .iter()
            .filter(|request| request.url == POPULAR_MOVIES_URL)
            .count(),
        1,
        "The spotlight and its row share one request"
    );
}

#[test]
fn spotlight_hides_the_catalog_row() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight {
        show_catalog_row: false,
        ..BoardSpotlight::default()
    });
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    assert_matches!(
        &board.spotlight,
        Some(ResourceLoadable { request, .. }) if *request == cinemeta_request("movie", "top")
    );
    assert!(
        board
            .catalogs
            .iter()
            .flat_map(|catalog| catalog.first())
            .all(|page| page.request != cinemeta_request("movie", "top")),
        "The spotlight catalog has no row"
    );
}

#[test]
fn spotlight_falls_back_to_the_first_catalog() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let missing_catalog = ResourceRequest::new(
        Url::parse("https://addon/manifest.json").unwrap(),
        ResourcePath::without_extra(CATALOG_RESOURCE_NAME, "movie", "missing"),
    );
    let (runtime, _rx) = runtime_with(BoardSpotlight {
        catalog: Some(missing_catalog.to_owned()),
        ..BoardSpotlight::default()
    });
    TestEnv::run(|| runtime.dispatch(load_board()));
    let model = runtime.model().unwrap();
    assert_matches!(
        &model.board.spotlight,
        Some(ResourceLoadable { request, .. }) if *request == cinemeta_request("movie", "top")
    );
    assert_eq!(
        model.ctx.profile.settings.board_spotlight.catalog,
        Some(missing_catalog),
        "The setting is kept for when the addon is installed again"
    );
}

#[test]
fn spotlight_disabled() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight {
        enabled: false,
        show_catalog_row: false,
        ..BoardSpotlight::default()
    });
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    assert!(board.spotlight.is_none());
    assert_matches!(
        board.catalogs.first().and_then(|catalog| catalog.first()),
        Some(ResourceLoadable { request, content: None }) if *request == cinemeta_request("movie", "top")
    );
    assert!(REQUESTS.read().unwrap().is_empty());
}
