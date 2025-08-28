use std::{
    any::{type_name, Any},
    collections::{BTreeMap, HashMap},
    ops::Fn,
    sync::{Arc, LockResult, Mutex, MutexGuard, RwLock},
};

use chrono::{DateTime, Utc};
use enclose::enclose;
use futures::{channel::mpsc::Receiver, future, Future, StreamExt, TryFutureExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    models::{ctx::Ctx, streaming_server::StreamingServer},
    runtime::{Env, EnvFuture, EnvFutureExt, Model, Runtime, RuntimeEvent, TryEnvFuture},
};

pub static FETCH_HANDLER: Lazy<RwLock<FetchHandler>> =
    Lazy::new(|| RwLock::new(Box::new(default_fetch_handler)));
pub static REQUESTS: Lazy<RwLock<Vec<Request>>> = Lazy::new(Default::default);
pub static STORAGE: Lazy<RwLock<BTreeMap<String, String>>> = Lazy::new(Default::default);
pub static EVENTS: Lazy<RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>> =
    Lazy::new(Default::default);
pub static STATES: Lazy<RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>> =
    Lazy::new(Default::default);
pub static NOW: Lazy<RwLock<DateTime<Utc>>> = Lazy::new(|| RwLock::new(Utc::now()));
pub static ENV_MUTEX: Mutex<()> = Mutex::new(());

pub type FetchHandler =
    Box<dyn Fn(Request) -> TryEnvFuture<Box<dyn Any + Send>> + Send + Sync + 'static>;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub enum TestEnv {}

impl TestEnv {
    pub fn reset() -> LockResult<MutexGuard<'static, ()>> {
        let env_mutex = ENV_MUTEX.lock();
        *FETCH_HANDLER.write().unwrap() = Box::new(default_fetch_handler);
        *REQUESTS.write().unwrap() = vec![];
        *STORAGE.write().unwrap() = BTreeMap::new();
        *EVENTS.write().unwrap() = vec![];
        *STATES.write().unwrap() = vec![];
        *NOW.write().unwrap() = Utc::now();
        env_mutex
    }
    pub fn run<F: FnOnce()>(runnable: F) {
        tokio_current_thread::block_on_all(future::lazy(|_| {
            runnable();
        }))
    }
    pub fn run_with_runtime<M: Model<TestEnv> + Clone + Send + Sync + 'static, F: FnOnce()>(
        rx: Receiver<RuntimeEvent<TestEnv, M>>,
        runtime: Arc<RwLock<Runtime<TestEnv, M>>>,
        runnable: F,
    ) {
        tokio_current_thread::block_on_all(future::lazy(|_| {
            {
                let runtime = runtime.read().expect("runtime read failed");
                let state = runtime.model().expect("model read failed");
                let mut states = STATES.write().expect("states write failed");
                states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
            }
            runnable();
        }));
        tokio_current_thread::block_on_all(future::lazy(|_| {
            TestEnv::exec_concurrent(rx.for_each(move |event| {
                if let RuntimeEvent::NewState(_, state) = &event {
                    let mut states = STATES.write().expect("states write failed");
                    states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
                };
                let mut events = EVENTS.write().expect("events write failed");
                events.push(Box::new(event) as Box<dyn Any + Send + Sync>);
                future::ready(())
            }));
            TestEnv::exec_concurrent(enclose!((runtime) async move {
                let mut runtime = runtime.write().expect("runtime read failed");
                runtime.close().await.unwrap();
            }));
        }));
    }
}

impl Env for TestEnv {
    fn fetch<IN: Serialize + 'static, OUT: for<'de> Deserialize<'de> + 'static>(
        request: http::Request<IN>,
    ) -> TryEnvFuture<OUT> {
        let request = Request::from(request);
        REQUESTS.write().unwrap().push(request.to_owned());
        FETCH_HANDLER.read().unwrap()(request)
            .map_ok(|resp| {
                *resp
                    .downcast::<OUT>()
                    .unwrap_or_else(|_| panic!("Failed to downcast to {}", type_name::<OUT>()))
            })
            .boxed_env()
    }
    fn get_storage<T: for<'de> Deserialize<'de> + 'static>(key: &str) -> TryEnvFuture<Option<T>> {
        future::ok(
            STORAGE
                .read()
                .unwrap()
                .get(key)
                .map(|data| serde_json::from_str(data).unwrap()),
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
    fn exec_concurrent<F: Future<Output = ()> + 'static>(future: F) {
        tokio_current_thread::spawn(future);
    }
    fn exec_sequential<F: Future<Output = ()> + 'static>(future: F) {
        tokio_current_thread::spawn(future);
    }
    fn now() -> DateTime<Utc> {
        *NOW.read().unwrap()
    }
    fn flush_analytics() -> EnvFuture<'static, ()> {
        future::ready(()).boxed_env()
    }
    fn analytics_context(
        _ctx: &Ctx,
        _streaming_server: &StreamingServer,
        _path: &str,
    ) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn log(message: String) {
        println!("{message}")
    }
}

pub fn default_fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
    panic!("Unhandled fetch request: {:#?}", request)
}
