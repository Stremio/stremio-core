use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::addon::{Descriptor, ResourceResponse};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::notifications::{NotificationItem, NotificationsBucket};
use crate::types::profile::Profile;
use crate::unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER};
use enclose::enclose;
use futures::future;
use serde::Deserialize;
use std::any::Any;
use std::collections::HashMap;
use stremio_derive::Model;

pub const DATA: &'static [u8] = include_bytes!("./data.json");

#[derive(Deserialize)]
struct TestData {
    network_requests: HashMap<String, ResourceResponse>,
    addons: Vec<Descriptor>,
    library_items: Vec<LibraryItem>,
    notification_items: Vec<NotificationItem>,
    result: Vec<NotificationItem>,
}

#[test]
fn notifications() {
    let tests = serde_json::from_slice::<Vec<TestData>>(DATA).unwrap();
    for test in tests {
        #[derive(Model)]
        #[model(TestEnv)]
        struct TestModel {
            ctx: Ctx,
        }
        let fetch_handler = enclose!((test.network_requests => network_requests) move |request: Request| -> TryEnvFuture<Box<dyn Any + Send>> {
            if let Some(result) = network_requests.get(&request.url) {
                return future::ok(Box::new(result.to_owned()) as Box<dyn Any + Send>).boxed_env();
            }

            return default_fetch_handler(request);
        });
        let _env_mutex = TestEnv::reset();
        *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
        let (runtime, _rx) = Runtime::<TestEnv, _>::new(
            TestModel {
                ctx: Ctx::new(
                    Profile {
                        addons: test.addons,
                        ..Default::default()
                    },
                    LibraryBucket::new(None, test.library_items),
                    NotificationsBucket::new::<TestEnv>(None, test.notification_items),
                ),
            },
            vec![],
            1000,
        );
        TestEnv::run(|| {
            runtime.dispatch(RuntimeAction {
                field: None,
                action: Action::Ctx(ActionCtx::PullNotificatons),
            })
        });
        assert_eq!(
            runtime
                .model()
                .unwrap()
                .ctx
                .notifications
                .items
                .values()
                .collect::<Vec<_>>(),
            test.result.iter().collect::<Vec<_>>(),
            "Notifications items match"
        );
    }
}
