use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Effects, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, LinkDataResponse};
use crate::unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn read_link_data() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://link.stremio.com/api/read?type=Read&code=CODE"
                && method == "GET"
                && body == "null" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: LinkDataResponse::AuthKey {
                        auth_key: "AUTH_KEY".to_owned(),
                        link: "LINK".to_owned(),
                        qrcode: "QRCODE".to_owned(),
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    TestEnv::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _rx) =
        Runtime::<TestEnv, _>::new(TestModel::default(), Effects::none().unchanged(), 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::ReadLinkCode("CODE".to_owned())),
        })
    });
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://link.stremio.com/api/read?type=Read&code=CODE".to_owned(),
            method: "GET".to_owned(),
            body: "null".to_owned(),
            ..Default::default()
        },
        "read request has been sent"
    );
}
