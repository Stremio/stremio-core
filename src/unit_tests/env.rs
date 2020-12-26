use crate::runtime::{Env, EnvFuture};
use chrono::{DateTime, Utc};
use futures::{future, Future, FutureExt, TryFutureExt};
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

pub type FetchHandler = Box<dyn Fn(Request) -> EnvFuture<Box<dyn Any>> + Send + Sync + 'static>;

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
    fn fetch<IN, OUT>(request: http::Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize,
        for<'de> OUT: Deserialize<'de> + 'static,
    {
        let request = Request::from(request);
        REQUESTS.write().unwrap().push(request.to_owned());
        FETCH_HANDLER.read().unwrap()(request)
            .map_ok(|resp| *resp.downcast::<OUT>().unwrap())
            .boxed_local()
    }
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        future::ok(
            STORAGE
                .read()
                .unwrap()
                .get(key)
                .map(|data| serde_json::from_str(&data).unwrap()),
        )
        .boxed_local()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
        let mut storage = STORAGE.write().unwrap();
        match value {
            Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
            None => storage.remove(key),
        };
        future::ok(()).boxed_local()
    }
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        tokio_current_thread::spawn(future);
    }
    fn now() -> DateTime<Utc> {
        *NOW.read().unwrap()
    }
    fn log(message: String) {
        println!("{}", message)
    }
    fn analytics_context() -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub fn default_fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
    panic!("Unhandled fetch request: {:#?}", request)
}
