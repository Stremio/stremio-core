use std::any::Any;

use futures::future;
use stremio_derive::Model;

use crate::{
    models::ctx::Ctx,
    runtime::{
        msg::{Action, ActionCtx},
        EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture,
    },
    types::api::{APIResult, GetModalResponse, GetNotificationResponse},
    unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER},
};

#[test]
fn test_events() {
    #[derive(Model, Clone, Debug)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }

    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request { url, .. } if url == "https://api.strem.io/api/getModal" => {
                future::ok(Box::new(APIResult::Ok {
                    result: GetModalResponse {
                        id: "id".to_owned(),
                        title: "title".to_owned(),
                        message: "message".to_owned(),
                        image_url: "https://image_url".parse().unwrap(),
                        addon: None,
                        external_url: None,
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request { url, .. } if url == "https://api.strem.io/api/getNotification" => {
                future::ok(Box::new(APIResult::Ok {
                    result: GetNotificationResponse {
                        id: "id".to_owned(),
                        title: "title".to_owned(),
                        message: "message".to_owned(),
                        external_url: None,
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }

    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");

    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);

    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::default(),
        },
        vec![],
        1000,
    );

    assert!(
        runtime.model().unwrap().ctx.events.modal.is_loading(),
        "Modal should be loading"
    );

    assert!(
        runtime
            .model()
            .unwrap()
            .ctx
            .events
            .notification
            .is_loading(),
        "Notification should be loading"
    );

    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::GetEvents),
        })
    });

    assert!(
        runtime.model().unwrap().ctx.events.modal.is_ready(),
        "Modal should be ready"
    );

    assert!(
        runtime.model().unwrap().ctx.events.notification.is_ready(),
        "Notification should be ready"
    );
}
