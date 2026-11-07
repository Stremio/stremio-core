use crate::constants::CATALOG_RESOURCE_NAME;
use crate::models::catalogs_with_extra::{CatalogsWithExtra, Selected};
use crate::models::common::{
    board_spotlight_requests, spotlight_items, CatalogPage, Loadable, ResourceError,
    ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad};
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
    board: CatalogsWithExtra,
}

const POPULAR_MOVIES_URL: &str = "https://v3-cinemeta.strem.io/catalog/movie/top.json";
const POPULAR_SERIES_URL: &str = "https://v3-cinemeta.strem.io/catalog/series/top.json";

fn meta(id: &str) -> MetaItemPreview {
    MetaItemPreview {
        id: id.to_owned(),
        background: Some(Url::parse("https://background").unwrap()),
        ..Default::default()
    }
}

fn meta_without_artwork(id: &str) -> MetaItemPreview {
    MetaItemPreview {
        id: id.to_owned(),
        ..Default::default()
    }
}

fn respond(metas: Vec<MetaItemPreview>) -> TryEnvFuture<Box<dyn Any + Send>> {
    future::ok(Box::new(ResourceResponse::Metas { metas }) as Box<dyn Any + Send>).boxed_env()
}

fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, method, .. } if url == POPULAR_MOVIES_URL && method == "GET" => {
            respond(vec![meta_without_artwork("tt0"), meta("tt1")])
        }
        Request { url, method, .. } if url == POPULAR_SERIES_URL && method == "GET" => {
            respond(vec![meta("tt2")])
        }
        _ => default_fetch_handler(request),
    }
}

fn fetch_handler_series_without_artwork(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, method, .. } if url == POPULAR_MOVIES_URL && method == "GET" => {
            respond(vec![meta("tt1")])
        }
        Request { url, method, .. } if url == POPULAR_SERIES_URL && method == "GET" => {
            respond(vec![meta_without_artwork("tt2")])
        }
        _ => default_fetch_handler(request),
    }
}

fn fetch_handler_series_fails(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request { url, method, .. } if url == POPULAR_MOVIES_URL && method == "GET" => {
            respond(vec![meta("tt1")])
        }
        Request { url, method, .. } if url == POPULAR_SERIES_URL && method == "GET" => {
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

fn missing_request() -> ResourceRequest {
    ResourceRequest::new(
        Url::parse("https://addon/manifest.json").unwrap(),
        ResourcePath::without_extra(CATALOG_RESOURCE_NAME, "movie", "missing"),
    )
}

fn runtime_with(
    board_spotlight: BoardSpotlight,
    fetch_handler: fn(Request) -> TryEnvFuture<Box<dyn Any + Send>>,
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

fn requests_to(url: &str) -> usize {
    REQUESTS
        .read()
        .unwrap()
        .iter()
        .filter(|request| request.url == url)
        .count()
}

fn page(
    r#type: &str,
    content: Option<Loadable<Vec<MetaItemPreview>, ResourceError>>,
) -> CatalogPage<MetaItemPreview> {
    ResourceLoadable {
        request: cinemeta_request(r#type, "top"),
        content,
    }
}

fn ready(r#type: &str, ids: &[&str]) -> CatalogPage<MetaItemPreview> {
    page(
        r#type,
        Some(Loadable::Ready(ids.iter().map(|id| meta(id)).collect())),
    )
}

fn slide_ids(pages: &[CatalogPage<MetaItemPreview>]) -> Vec<String> {
    spotlight_items(pages.iter())
        .into_iter()
        .map(|(_, item)| item.id.to_owned())
        .collect()
}

fn board_requests(board: &CatalogsWithExtra) -> Vec<&ResourceRequest> {
    board
        .catalogs
        .iter()
        .flat_map(|catalog| catalog.first())
        .map(|page| &page.request)
        .collect()
}

fn spotlight_requests(board: &CatalogsWithExtra) -> Vec<ResourceRequest> {
    board
        .spotlight
        .iter()
        .map(|page| page.request.to_owned())
        .collect()
}

#[test]
fn spotlight_defaults_to_movies_and_series() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(BoardSpotlight::default(), fetch_handler);
    TestEnv::run(|| {
        runtime.dispatch(load_board());
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadRange(0..1)),
        });
    });
    let board = &runtime.model().unwrap().board;
    assert_eq!(
        spotlight_requests(board),
        vec![
            cinemeta_request("movie", "top"),
            cinemeta_request("series", "top")
        ]
    );
    assert_eq!(slide_ids(&board.spotlight), vec!["tt1", "tt2"]);
    assert_matches!(
        board.catalogs.first().and_then(|catalog| catalog.first()),
        Some(ResourceLoadable {
            request,
            content: Some(Loadable::Ready(items)),
        }) if *request == cinemeta_request("movie", "top") && items.len() == 2
    );
    assert_eq!(
        requests_to(POPULAR_MOVIES_URL),
        1,
        "The spotlight and its row share one request"
    );
    assert_eq!(requests_to(POPULAR_SERIES_URL), 1);
}

#[test]
fn spotlight_default_without_movies_or_series() {
    let mut profile = Profile::default();
    profile
        .addons
        .retain(|addon| addon.transport_url.as_str().contains("v3-channels"));
    let requests = board_spotlight_requests(&profile);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path.r#type, "channel");
}

#[test]
fn spotlight_hides_the_catalog_rows() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            show_catalog_rows: false,
            ..BoardSpotlight::default()
        },
        fetch_handler,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    let requests = board_requests(board);
    assert!(!requests.contains(&&cinemeta_request("movie", "top")));
    assert!(!requests.contains(&&cinemeta_request("series", "top")));
}

#[test]
fn spotlight_row_returns_without_artwork() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            show_catalog_rows: false,
            ..BoardSpotlight::default()
        },
        fetch_handler_series_without_artwork,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    let requests = board_requests(board);
    assert!(!requests.contains(&&cinemeta_request("movie", "top")));
    assert_eq!(requests[0], &cinemeta_request("series", "top"));
    assert_matches!(
        board.catalogs[0].first(),
        Some(ResourceLoadable {
            content: Some(Loadable::Ready(items)),
            ..
        }) if items.len() == 1
    );
    assert_eq!(
        requests_to(POPULAR_SERIES_URL),
        1,
        "The returned row reuses the spotlight page"
    );
}

#[test]
fn spotlight_row_returns_when_a_pick_fails() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            show_catalog_rows: false,
            ..BoardSpotlight::default()
        },
        fetch_handler_series_fails,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    assert_eq!(slide_ids(&board.spotlight), vec!["tt1"]);
    assert_eq!(board_requests(board)[0], &cinemeta_request("series", "top"));
}

#[test]
fn spotlight_ignores_missing_picks() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let catalogs = vec![missing_request(), cinemeta_request("series", "top")];
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            catalogs: catalogs.to_owned(),
            ..BoardSpotlight::default()
        },
        fetch_handler,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    let model = runtime.model().unwrap();
    assert_eq!(
        spotlight_requests(&model.board),
        vec![cinemeta_request("series", "top")]
    );
    assert_eq!(
        model.ctx.profile.settings.board_spotlight.catalogs, catalogs,
        "The setting is kept for when the addon is installed again"
    );
}

#[test]
fn spotlight_falls_back_to_the_default() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            catalogs: vec![missing_request()],
            ..BoardSpotlight::default()
        },
        fetch_handler,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    assert_eq!(
        spotlight_requests(&runtime.model().unwrap().board),
        vec![
            cinemeta_request("movie", "top"),
            cinemeta_request("series", "top")
        ]
    );
}

#[test]
fn spotlight_limits_the_picks() {
    let mut profile = Profile::default();
    profile.settings.board_spotlight.catalogs = vec![
        cinemeta_request("series", "top"),
        cinemeta_request("movie", "top"),
        cinemeta_request("movie", "imdbRating"),
        cinemeta_request("series", "imdbRating"),
    ];
    assert_eq!(
        board_spotlight_requests(&profile),
        profile.settings.board_spotlight.catalogs[..3]
    );
}

#[test]
fn spotlight_disabled() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let (runtime, _rx) = runtime_with(
        BoardSpotlight {
            enabled: false,
            show_catalog_rows: false,
            ..BoardSpotlight::default()
        },
        fetch_handler,
    );
    TestEnv::run(|| runtime.dispatch(load_board()));
    let board = &runtime.model().unwrap().board;
    assert!(board.spotlight.is_empty());
    assert_matches!(
        board.catalogs.first().and_then(|catalog| catalog.first()),
        Some(ResourceLoadable { request, content: None }) if *request == cinemeta_request("movie", "top")
    );
    assert!(REQUESTS.read().unwrap().is_empty());
}

#[test]
fn spotlight_items_take_turns() {
    let many = ["a1", "a2", "a3", "a4", "a5", "a6", "a7"];
    assert_eq!(
        slide_ids(&[ready("movie", &many)]),
        vec!["a1", "a2", "a3", "a4", "a5", "a6"],
        "A single pick keeps its own order"
    );
    assert_eq!(
        slide_ids(&[
            ready("movie", &many),
            ready("series", &["b1", "b2", "b3", "b4"])
        ]),
        vec!["a1", "b1", "a2", "b2", "a3", "b3"]
    );
    assert_eq!(
        slide_ids(&[
            ready("movie", &many),
            ready("series", &["b1", "b2", "b3"]),
            ready("channel", &["c1", "c2", "c3"]),
        ]),
        vec!["a1", "b1", "c1", "a2", "b2", "c2"]
    );
}

#[test]
fn spotlight_items_skip_duplicates() {
    assert_eq!(
        slide_ids(&[
            ready("movie", &["a1", "shared", "a2"]),
            ready("series", &["shared", "b1"]),
        ]),
        vec!["a1", "shared", "a2", "b1"]
    );
}

#[test]
fn spotlight_items_hand_over_empty_turns() {
    assert_eq!(
        slide_ids(&[
            ready("movie", &["a1"]),
            ready("series", &["b1", "b2", "b3", "b4", "b5", "b6"]),
        ]),
        vec!["a1", "b1", "b2", "b3", "b4", "b5"]
    );
}

#[test]
fn spotlight_items_wait_for_earlier_picks() {
    let loading = || page("series", Some(Loadable::Loading));
    let failed = || page("series", Some(Loadable::Err(ResourceError::EmptyContent)));
    assert!(slide_ids(&[loading(), ready("movie", &["a1"])]).is_empty());
    assert_eq!(
        slide_ids(&[
            ready("movie", &["a1"]),
            loading(),
            ready("channel", &["c1"])
        ]),
        vec!["a1"],
        "A slow pick never lets a later one in front"
    );
    assert_eq!(
        slide_ids(&[failed(), ready("movie", &["a1"])]),
        vec!["a1"],
        "A failed pick is skipped"
    );
}

#[test]
fn spotlight_setting_ignores_the_old_keys() {
    let board_spotlight = serde_json::from_value::<BoardSpotlight>(serde_json::json!({
        "enabled": false,
        "catalog": {
            "base": "https://v3-cinemeta.strem.io/manifest.json",
            "path": { "resource": "catalog", "type": "movie", "id": "top", "extra": [] },
        },
        "showCatalogRow": false,
    }))
    .expect("A stored single catalog must not break the profile");
    assert_eq!(
        board_spotlight,
        BoardSpotlight {
            enabled: false,
            ..BoardSpotlight::default()
        }
    );
}
