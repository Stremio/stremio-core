use crate::models::common::Loadable;
use crate::models::ctx::Ctx;
use crate::models::data_export::DataExport;
use crate::runtime::msg::{Action, ActionLoad};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, RuntimeEvent, TryEnvFuture};
use crate::types::api::{APIResult, DataExportResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::LibraryBucket;
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::Profile;
use crate::types::profile::{Auth, AuthKey, User};
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

#[derive(Model, Clone, Debug)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    data_export: DataExport,
}

fn data_export_fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    match request {
        Request {
            url, method, body, ..
        } if url == "https://api.strem.io/api/dataExport"
            && method == "POST"
            && &body == r#"{"type":"DataExport","authKey":"user_key"}"# =>
        {
            future::ok(Box::new(APIResult::Ok(DataExportResponse {
                export_id: "user_export_id".into(),
            })) as Box<dyn Any + Send>)
            .boxed_env()
        }
        _ => default_fetch_handler(request),
    }
}

#[test]
fn data_export_with_user() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(data_export_fetch_handler);
    let mut ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    ctx.profile.auth = Some(Auth {
        key: AuthKey("user_key".into()),
        user: User::default(),
    });

    let data_export = DataExport::default();
    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel { ctx, data_export }, vec![], 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::DataExport),
            });
        }),
    );
    let events = EVENTS.read().unwrap();
    assert_eq!(events.len(), 2);

    assert_matches!(
        events[0]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::DataExport
    );
    assert_matches!(
        events[1]
            .downcast_ref::<RuntimeEvent<TestEnv, TestModel>>()
            .unwrap(),
        RuntimeEvent::NewState(fields, _) if fields.len() == 1 && *fields.first().unwrap() == TestModelField::DataExport
    );
    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(states.len(), 3);
    assert!(states[0].data_export.export_url.is_none());
    assert_eq!(
        &states[1].data_export.export_url,
        &Some((AuthKey("user_key".into()), Loadable::Loading))
    );

    assert_eq!(
        &states[2].data_export.export_url,
        &Some((
            AuthKey("user_key".into()),
            Loadable::Ready(
                "https://api.strem.io/data-export/user_export_id/export.json"
                    .parse()
                    .unwrap()
            )
        ))
    );
    let requests = REQUESTS.read().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0],
        Request {
            url: "https://api.strem.io/api/dataExport".to_owned(),
            method: "POST".to_owned(),
            headers: Default::default(),
            body: r#"{"type":"DataExport","authKey":"user_key"}"#.to_owned(),
        }
    );
}

#[test]
fn data_export_without_a_user() {
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(data_export_fetch_handler);
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );

    assert!(
        ctx.profile.auth.is_none(),
        "For this test we require no authenticated user!"
    );

    let data_export = DataExport::default();
    let (runtime, rx) = Runtime::<TestEnv, _>::new(TestModel { ctx, data_export }, vec![], 1000);
    let runtime = Arc::new(RwLock::new(runtime));
    TestEnv::run_with_runtime(
        rx,
        runtime.clone(),
        enclose!((runtime) move || {
            let runtime = runtime.read().unwrap();
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Load(ActionLoad::DataExport),
            });
        }),
    );
    let events = EVENTS.read().unwrap();
    assert!(events.is_empty());

    let states = STATES.read().unwrap();
    let states = states
        .iter()
        .map(|state| state.downcast_ref::<TestModel>().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(states.len(), 1);
    assert!(states[0].data_export.export_url.is_none());

    let requests = REQUESTS.read().unwrap();
    assert!(requests.is_empty());
}
