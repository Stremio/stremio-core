use crate::models::common::Loadable;
use crate::models::ctx::Ctx;
use crate::models::link::Link;
use crate::runtime::msg::{Action, ActionLink};
use crate::runtime::{Effects, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, LinkAuthKey, LinkCodeResponse, LinkDataResponse};
use crate::unit_tests::{default_fetch_handler, Request, TestEnv, FETCH_HANDLER, REQUESTS};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn create_link_code() {
    #[derive(Model, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
        link: Link<LinkAuthKey>,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://link.stremio.com/api/create?type=Create"
                && method == "GET"
                && body == "null" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: LinkCodeResponse {
                        code: "CODE".to_owned(),
                        link: "LINK".to_owned(),
                        qrcode: "QRCODE".to_owned(),
                    },
                }) as Box<dyn Any + Send>)
                .boxed_env()
            }
            Request {
                url, method, body, ..
            } if url == "https://link.stremio.com/api/read?type=Read&code=CODE"
                && method == "GET"
                && body == "null" =>
            {
                future::ok(Box::new(APIResult::Ok {
                    result: LinkDataResponse::AuthKey(LinkAuthKey {
                        auth_key: "AUTH_KEY".to_owned(),
                    }),
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
            action: Action::Link(ActionLink::CreateCode),
        })
    });
    assert_eq!(
        runtime.model().unwrap().link.code,
        Some(Loadable::Ready(LinkCodeResponse {
            code: "CODE".to_owned(),
            link: "LINK".to_owned(),
            qrcode: "QRCODE".to_owned(),
        })),
        "link code loaded"
    );
    assert_eq!(
        runtime.model().unwrap().link.data,
        None,
        "link data not loaded"
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Link(ActionLink::ReadData),
        })
    });
    assert_eq!(
        runtime.model().unwrap().link.code,
        Some(Loadable::Ready(LinkCodeResponse {
            code: "CODE".to_owned(),
            link: "LINK".to_owned(),
            qrcode: "QRCODE".to_owned(),
        })),
        "link code loaded"
    );
    assert_eq!(
        runtime.model().unwrap().link.data,
        Some(Loadable::Ready(LinkAuthKey {
            auth_key: "AUTH_KEY".to_owned(),
        })),
        "link data loaded"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        2,
        "Two requests have been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().to_owned(),
        Request {
            url: "https://link.stremio.com/api/create?type=Create".to_owned(),
            method: "GET".to_owned(),
            body: "null".to_owned(),
            ..Default::default()
        },
        "create request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(1).unwrap().to_owned(),
        Request {
            url: "https://link.stremio.com/api/read?type=Read&code=CODE".to_owned(),
            method: "GET".to_owned(),
            body: "null".to_owned(),
            ..Default::default()
        },
        "read request has been sent"
    );
}
