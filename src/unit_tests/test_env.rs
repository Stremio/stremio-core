use std::{
    any::{type_name, Any},
    collections::{BTreeMap, HashMap},
    fmt,
    ops::Fn,
    sync::{Arc, LockResult, Mutex, MutexGuard, RwLock},
    time::Duration,
};

use chrono::{DateTime, Utc};
use enclose::enclose;
use futures::{channel::mpsc::Receiver, future, Future, StreamExt, TryFutureExt};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::{
    models::{ctx::Ctx, streaming_server::StreamingServer},
    runtime::{
        ConditionalSend, Env, EnvFuture, EnvFutureExt, Model, Runtime, RuntimeEvent, TryEnvFuture,
    },
};


use super::{default_fetch_handler, FetchHandler, Request};

pub static TEST_ENV: Lazy<TestEnv> = Lazy::new(|| TestEnv::new());

#[derive(Clone, Debug)]
pub struct TestEnv {
    pub inner: Arc<TestEnvInner>,
}

pub struct TestEnvInner {
    pub(crate) fetch_handler: RwLock<FetchHandler>,
    pub(crate) requests: RwLock<Vec<Request>>,
    pub(crate) storage: RwLock<BTreeMap<String, String>>,
    pub(crate) events: RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>,
    pub(crate) states: RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>,
    pub(crate) now: RwLock<DateTime<Utc>>,
    pub(crate) env_mutex: Mutex<()>,
    // runtime: tokio::runtime::Runtime,
}

impl fmt::Debug for TestEnvInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEnvInner")
            .field("fetch_handler", &"FetchHandler")
            .field("requests", &self.requests)
            .field("storage", &self.storage)
            .field("events", &self.events)
            .field("states", &self.states)
            .field("now", &self.now)
            .field("env_mutex", &self.env_mutex)
            .finish()
    }
}

impl TestEnv {
    fn new() -> Self {
        Self {
            inner: Arc::new(TestEnvInner {
                fetch_handler: RwLock::new(Box::new(default_fetch_handler)),
                requests: Default::default(),
                storage: Default::default(),
                events: Default::default(),
                states: Default::default(),
                now: RwLock::new(Utc::now()),
                env_mutex: Default::default(),
            }),
        }
    }

    pub fn set_fetch_handler(&self, new: FetchHandler) {
        let mut guard = self.inner.fetch_handler.write().unwrap();
        *guard = new;

        // self
    }
}

pub struct TestGuard {
    receivers: JoinHandle<()>,
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        self.receivers.abort();
    }
}

// pub struct EnvGuard<M: Model<TestEnv> + Send + Sync + 'static> {
//     runtime: Arc<RwLock<Runtime<TestEnv, M>>>,
// }

// impl<M> Drop for EnvGuard<M>
// where
//     M: Model<TestEnv> + Send + Sync + 'static,
// {
//     fn drop(&mut self) {
//         futures::executor::block_on(async move {
//             let mut runtime = self.runtime.write().expect("runtime write failed");
//             println!("run_with_runtime: runtime.close()");
//             runtime.close().await.unwrap();
//         });

//         println!("Dropped TestEnv");
//     }
// }

impl TestEnv {
    pub fn reset(&'static self) -> LockResult<MutexGuard<'static, ()>> {
        let env_mutex = self.inner.env_mutex.lock();
        *self.inner.fetch_handler.write().unwrap() = Box::new(default_fetch_handler);
        self.inner.requests.write().unwrap().clear();
        self.inner.storage.write().unwrap().clear();
        self.inner.events.write().unwrap().clear();
        self.inner.states.write().unwrap().clear();
        *self.inner.now.write().unwrap() = Utc::now();
        env_mutex
    }
    pub fn run<F: FnOnce()>(runnable: F) {
        futures::executor::block_on(future::lazy(|_| {
            // tokio_current_thread::block_on_all(future::lazy(|_| {
            runnable();
        }))
    }

    // pub fn run_with_runtime_futures<
    //     M: Model<TestEnv> + core::fmt::Debug + Clone + Send + Sync + 'static,
    //     F: FnOnce(),
    // >(
    //     &'static self,
    //     rx: Receiver<RuntimeEvent<TestEnv, M>>,
    //     runtime: Arc<RwLock<Runtime<TestEnv, M>>>,
    //     runnable: F,
    //     // ) -> EnvGuard<M> {
    // ) {
    //     // tokio_current_thread::block_on_all(future::lazy(|_| {
    //     futures::executor::block_on(future::lazy(|_| {
    //         TestEnv::exec_concurrent(rx.for_each(enclose!((runtime) move |event| {
    //             println!("run_with_runtime: event");
    //             if let RuntimeEvent::NewState(_) = event {
    //                 let runtime = runtime.read().expect("runtime read failed");
    //                 let state = runtime.model().expect("model read failed");
    //                 let mut states = self.inner.states.write().expect("states write failed");
    //                 states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
    //             };
    //             let mut events = TEST_ENV.inner.events.write().expect("events write failed");
    //             events.push(Box::new(event) as Box<dyn Any + Send + Sync>);
    //             println!("run_with_runtime: pushed");

    //             future::ready(())
    //         })));
    //         {
    //             let runtime = runtime.read().expect("runtime read failed");
    //             let state = runtime.model().expect("model read failed");
    //             let mut states = self.inner.states.write().expect("states write failed");
    //             states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
    //         }
    //         println!("run_with_runtime: before runnable");
    //         runnable();
    //         println!("run_with_runtime: after runnable");
    //         // TestEnv::exec_concurrent(enclose!((runtime) async move {
    //         //     let mut runtime = runtime.write().expect("runtime read failed");
    //         //     println!("run_with_runtime: runtime.close()");
    //         //     runtime.close().await.unwrap();
    //         // }));
    //     }))
    // }

    pub async fn run_with_runtime_tokio<
        M: Model<TestEnv> + Clone + Send + Sync + 'static + fmt::Debug,
        F: FnOnce(),
    >(
        &self,
        mut rx: Receiver<RuntimeEvent<TestEnv, M>>,
        runtime: Arc<RwLock<Runtime<TestEnv, M>>>,
        runnable: F,
        // ) -> EnvGuard<M> {
    ) -> TestGuard {
        let receivers_envs = self.clone();

        {
            let runtime = runtime.read().expect("runtime read failed");
            let state = runtime.model().expect("model read failed");
            let mut states = TEST_ENV.inner.states.write().expect("states write failed");
            states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
        }

        let receivers = tokio::task::spawn(async move {
            let runtime = runtime.clone();

            println!("Receiver for new events has been spawned.");

            while let Some(event) = rx.next().await {
                // rx.for_each(enclose!((runtime, receivers_envs) move |event| {
                println!("Received new");

                if let RuntimeEvent::NewState(_) = &event {
                    let runtime = runtime.read().expect("runtime read failed");
                    let state = runtime.model().expect("model read failed");
                    let mut states = receivers_envs
                        .inner
                        .states
                        .write()
                        .expect("states write failed");
                    states.push(Box::new(state.to_owned()) as Box<dyn Any + Send + Sync>);
                    tracing::debug!("NewState pushed to states");
                    println!("NewState pushed to states");
                };
                let mut events = receivers_envs
                    .inner
                    .events
                    .write()
                    .expect("events write failed");
                tracing::debug!("Event added - {:?}", event);
                println!("Event added - {:?}", event);
                events.push(Box::new(event) as Box<dyn Any + Send + Sync>);
            }
        });

        println!("run_with_runtime: before runnable");
        runnable();
        println!("run_with_runtime: after runnable");
        // receivers_runtime.write().expect("Lock").flush().await.expect("Flushed all pending events");

        // give some time to the tasks to update
        tokio::time::sleep(Duration::from_secs(1)).await;

        // })
        // // await the spawned future and block until it's ready
        // .await;
        // // });

        // futures::executor::block_on(future::lazy(enclose!((runtime) |_| {
        // tokio::block_on(future::lazy(enclose!((runtime) |_| {

        // only on drop!
        // let mut runtime = runtime.write().expect("runtime read failed");
        // println!("run_with_runtime: runtime.close()");
        // runtime.close().await.unwrap();
        TestGuard { receivers }
    }
}

impl Env for TestEnv {
    fn fetch<
        IN: Serialize + ConditionalSend + 'static,
        OUT: for<'de> Deserialize<'de> + ConditionalSend + 'static,
    >(
        request: http::Request<IN>,
    ) -> TryEnvFuture<OUT> {
        let request = Request::from(request);
        TEST_ENV
            .inner
            .requests
            .write()
            .unwrap()
            .push(request.to_owned());
        TEST_ENV.inner.fetch_handler.read().unwrap()(request)
            .map_ok(|resp| {
                *resp
                    .downcast::<OUT>()
                    .unwrap_or_else(|_| panic!("Failed to downcast to {}", type_name::<OUT>()))
            })
            .boxed_env()
    }
    fn get_storage<T: for<'de> Deserialize<'de> + ConditionalSend + 'static>(
        key: &str,
    ) -> TryEnvFuture<Option<T>> {
        future::ok(
            TEST_ENV
                .inner
                .storage
                .read()
                .unwrap()
                .get(key)
                .map(|data| serde_json::from_str(data).unwrap()),
        )
        .boxed_env()
    }
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let mut storage = TEST_ENV.inner.storage.write().unwrap();
        match value {
            Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
            None => storage.remove(key),
        };
        future::ok(()).boxed_env()
    }
    fn exec_concurrent<F: Future<Output = ()> + ConditionalSend + 'static>(future: F) {
        // tokio::spawn(future);
        futures::executor::block_on(future);
    }
    fn exec_sequential<F: Future<Output = ()> + ConditionalSend + 'static>(future: F) {
        futures::executor::block_on(future)
        // tokio::spawn(future);
    }
    fn now() -> DateTime<Utc> {
        TEST_ENV.inner.now.read().unwrap().clone()
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
