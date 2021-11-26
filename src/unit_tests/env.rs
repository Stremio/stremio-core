use crate::models::ctx::Ctx;
use crate::models::streaming_server::StreamingServer;
use crate::runtime::{Env, EnvFuture, EnvFutureExt, TryEnvFuture};
use chrono::{DateTime, Utc};
use futures::{future, Future, TryFutureExt};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Fn;
use std::sync::RwLock;

lazy_static! {
    pub static ref FETCH_HANDLER: RwLock<FetchHandler> =
        RwLock::new(Box::new(default_fetch_handler));
    pub static ref REQUESTS: RwLock<Vec<Request>> = Default::default();
    pub static ref STORAGE: RwLock<BTreeMap<String, String>> = Default::default();
    pub static ref NOW: RwLock<DateTime<Utc>> = RwLock::new(Utc::now());
}

pub type FetchHandler =
    Box<dyn Fn(Request) -> TryEnvFuture<Box<dyn Any + Send>> + Send + Sync + 'static>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl<T: Serialize> From<http::Request<T>> for Request {
    fn from(request: http::Request<T>) -> Self {
        let (head, body) = request.into_parts();
        Request {
            url: head.uri.to_string(),
            method: head.method.as_str().to_owned(),
            headers: head
                .headers
                .iter()
                .map(|(key, value)| (key.as_str().to_owned(), value.to_str().unwrap().to_owned()))
                .collect::<HashMap<_, _>>(),
            body: serde_json::to_string(&body).unwrap(),
        }
    }
}

pub enum TestEnv {}

impl TestEnv {
    pub fn reset() {
        *FETCH_HANDLER.write().unwrap() = Box::new(default_fetch_handler);
        *REQUESTS.write().unwrap() = vec![];
        *STORAGE.write().unwrap() = BTreeMap::new();
        *NOW.write().unwrap() = Utc::now();
    }
    pub fn run<F: FnOnce()>(runnable: F) {
        tokio_current_thread::block_on_all(future::lazy(|_| {
            runnable();
        }))
    }
}

impl Env for TestEnv {
    fn fetch<IN, OUT>(request: http::Request<IN>) -> TryEnvFuture<OUT>
    where
        IN: Serialize,
        for<'de> OUT: Deserialize<'de> + 'static,
    {
        let request = Request::from(request);
        REQUESTS.write().unwrap().push(request.to_owned());
        FETCH_HANDLER.read().unwrap()(request)
            .map_ok(|resp| *resp.downcast::<OUT>().unwrap())
            .boxed_env()
    }
    fn get_storage<
        #[cfg(target_arch = "wasm32")] T: for<'de> Deserialize<'de> + 'static,
        #[cfg(not(target_arch = "wasm32"))] T: for<'de> Deserialize<'de> + Send + 'static,
    >(
        key: &str,
    ) -> TryEnvFuture<Option<T>> {
        future::ok(
            STORAGE
                .read()
                .unwrap()
                .get(key)
                .map(|data| serde_json::from_str(&data).unwrap()),
        )
        .boxed_env()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let mut storage = STORAGE.write().unwrap();
        match value {
            Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
            None => storage.remove(key),
        };
        future::ok(()).boxed_env()
    }
    fn exec<F: Future<Output = ()> + 'static>(future: F) {
        tokio_current_thread::spawn(future);
    }
    fn now() -> DateTime<Utc> {
        *NOW.read().unwrap()
    }
    fn flush_analytics() -> EnvFuture<()> {
        future::ready(()).boxed_env()
    }
    fn analytics_context(_ctx: &Ctx, _streaming_server: &StreamingServer) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn log(message: String) {
        println!("{}", message)
    }
}

pub fn default_fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    panic!("Unhandled fetch request: {:#?}", request)
}
