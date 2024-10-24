use crate::constants::STREAMING_SERVER_URLS_STORAGE_KEY;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, Runtime, RuntimeAction};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::unit_tests::{TestEnv, STORAGE};
use stremio_derive::Model;
use url::Url;

#[test]
fn test_add_server_url() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);
    let new_url = Url::parse("http://localhost:11470").unwrap();
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::AddServerUrl(new_url.clone())),
        })
    });
    let server_urls = &runtime.model().unwrap().ctx.streaming_server_urls;
    assert!(
        server_urls.items.contains_key(&new_url),
        "New server URL should be added"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(STREAMING_SERVER_URLS_STORAGE_KEY)
            .map_or(false, |data| {
                let stored_bucket: ServerUrlsBucket = serde_json::from_str(data).unwrap();
                stored_bucket.items.contains_key(&new_url)
            }),
        "New server URL should be stored"
    );
}

#[test]
fn test_delete_server_url() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    // Initialize with a server URL
    let initial_url = Url::parse("http://localhost:11470").unwrap();
    let mut server_urls = ServerUrlsBucket::new::<TestEnv>(None);
    server_urls
        .items
        .insert(initial_url.clone(), TestEnv::now());

    STORAGE.write().unwrap().insert(
        STREAMING_SERVER_URLS_STORAGE_KEY.to_owned(),
        serde_json::to_string(&server_urls).unwrap(),
    );

    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        server_urls,
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(TestModel { ctx }, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::DeleteServerUrl(initial_url.clone())),
        })
    });
    let server_urls = &runtime.model().unwrap().ctx.streaming_server_urls;
    assert!(
        !server_urls.items.contains_key(&initial_url),
        "Server URL should be deleted"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(STREAMING_SERVER_URLS_STORAGE_KEY)
            .map_or(true, |data| {
                let stored_bucket: ServerUrlsBucket = serde_json::from_str(data).unwrap();
                !stored_bucket.items.contains_key(&initial_url)
            }),
        "Deleted server URL should not be stored"
    );
}
