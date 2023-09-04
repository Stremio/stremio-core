use std::{
    any::Any,
    collections::BTreeMap,
    sync::{Arc, LockResult, Mutex, MutexGuard, RwLock},
};

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use thiserror::Error;

use crate::runtime::Env;

use super::{default_fetch_handler, FetchHandler, Request};

static ENV: Lazy<TestEnv2> = Lazy::new(|| TestEnv2::new(None, None));

pub struct TestEnv2 {
    // used to lock the struct while the test is running
    env_mutex: Mutex<()>,
    pub fetch_handler: RwLock<FetchHandler>,
    pub requests: RwLock<Vec<Request>>,
    pub storage: RwLock<BTreeMap<String, String>>,
    pub events: RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>,
    pub states: RwLock<Vec<Box<dyn Any + Send + Sync + 'static>>>,
    pub now: RwLock<DateTime<Utc>>,
    pub runtime: tokio::runtime::Runtime,
    // pub fetch_handler: FetchHandler,
    // pub requests: Vec<Request>,
    // pub storage: BTreeMap<String, String>,
    // pub events: Vec<Box<dyn Any + Send + Sync + 'static>>,
    // pub states: Vec<Box<dyn Any + Send + Sync + 'static>>,
    // pub now: DateTime<Utc>,
}
impl TestEnv2 {
    pub fn new(fetcher: Option<FetchHandler>, now: impl Into<Option<DateTime<Utc>>>) -> Self {
        let fetcher: Option<FetchHandler> = fetcher.into();
        let now_datetime = now.into();
        Self {
            env_mutex: Default::default(),
            fetch_handler: RwLock::new(fetcher.unwrap_or(Box::new(default_fetch_handler))),
            requests: Default::default(),
            storage: Default::default(),
            events: Default::default(),
            states: Default::default(),
            now: RwLock::new(now_datetime.unwrap_or(Utc::now())),
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Should build tokio runtime"),
        }
    }

    // with Lazy static
    pub fn reset() -> LockResult<MutexGuard<'static, ()>> {
        let env_mutex = ENV.env_mutex.lock()?;

        *ENV.fetch_handler.write().unwrap() = Box::new(default_fetch_handler);
        *ENV.requests.write().unwrap() = vec![];
        *ENV.storage.write().unwrap() = BTreeMap::new();
        *ENV.events.write().unwrap() = vec![];
        *ENV.states.write().unwrap() = vec![];
        *ENV.now.write().unwrap() = Utc::now();
        
        Ok(env_mutex)

        // with Env lock
        // let mut env_lock = ENV.lock()?;
        // env_lock.fetch_handler = Box::new(default_fetch_handler);
        // env_lock.requests = Default::default();
        // env_lock.storage = Default::default();
        // env_lock.events = Default::default();
        // env_lock.states = Default::default();
        // env_lock.now = Utc::now();

        // Ok(env_lock)
    }

    fn set_now(&self, now: DateTime<Utc>) {
        *self.now.write().expect("Should lock Now") = now;
    }

    // /// resets the TestEnv and set's the values from `new`
    // pub fn reset_with(new: TestEnv2) -> LockResult<MutexGuard<'static, ()>> {
    //     let env_mutex = ENV.env_mutex.lock()?;

    //     *ENV.fetch_handler.write().unwrap() = *new.fetch_handler.read().unwrap();
    //     *ENV.requests.write().unwrap() = (*new.requests.read().unwrap()).clone();
    //     *ENV.storage.write().unwrap() = (*new.storage.read().unwrap()).clone();
    //     *ENV.states.write().unwrap() = (*new.states.read().unwrap()).clone();
    //     *ENV.events.write().unwrap() = (*new.events.read().unwrap()).clone();
    //     *ENV.now.write().unwrap() = *new.now.read().unwrap();
    //     Ok(env_mutex)
    // }
    // pub fn reset_with(new: TestEnv2) -> LockResult<MutexGuard<'static, TestEnv2>> {
    //     let mut env_lock = ENV.lock()?;
    //     env_lock.fetch_handler = new.fetch_handler;
    //     env_lock.requests = new.requests;
    //     env_lock.storage = new.storage;
    //     env_lock.events = new.events;
    //     env_lock.states = new.states;
    //     env_lock.now = new.now;

    //     Ok(env_lock)
    // }
}

impl Default for TestEnv2 {
    fn default() -> Self {
        Self::new(Some(Box::new(default_fetch_handler)), Utc::now())
    }
}

impl Env for TestEnv2 {
    fn fetch<
        IN: serde::Serialize + crate::runtime::ConditionalSend + 'static,
        OUT: for<'de> serde::Deserialize<'de> + crate::runtime::ConditionalSend + 'static,
    >(
        request: http::Request<IN>,
    ) -> crate::runtime::TryEnvFuture<OUT> {
        todo!()
    }

    fn get_storage<
        T: for<'de> serde::Deserialize<'de> + crate::runtime::ConditionalSend + 'static,
    >(
        key: &str,
    ) -> crate::runtime::TryEnvFuture<Option<T>> {
        todo!()
    }

    fn set_storage<T: serde::Serialize>(
        key: &str,
        value: Option<&T>,
    ) -> crate::runtime::TryEnvFuture<()> {
        todo!()
    }

    fn exec_concurrent<
        F: futures::Future<Output = ()> + crate::runtime::ConditionalSend + 'static,
    >(
        future: F,
    ) {
        todo!()
    }

    fn exec_sequential<
        F: futures::Future<Output = ()> + crate::runtime::ConditionalSend + 'static,
    >(
        future: F,
    ) {
        todo!()
    }

    fn now() -> DateTime<Utc> {
        *ENV.now.read().expect("Failed to read now")
    }

    fn flush_analytics() -> crate::runtime::EnvFuture<'static, ()> {
        todo!()
    }

    fn analytics_context(
        ctx: &crate::models::ctx::Ctx,
        streaming_server: &crate::models::streaming_server::StreamingServer,
        path: &str,
    ) -> serde_json::Value {
        todo!()
    }

    fn log(message: String) {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to lock local Env mutex")]
    EnvLock(#[from] std::sync::PoisonError<MutexGuard<'static, ()>>),
}

impl TestEnv2 {
    pub fn new_case() -> Result<(TestCaseEnv, MutexGuard<'static, ()>), Error> {
        // let env_mutex = ENV_MUTEX.with(|mutex| mutex.lock())?;

        // Ok((TestCaseEnv::default(), env_mutex))
        todo!("Does not work with thread_local because of lifetime")
    }
}

// #[derive(Debug, PartialEq, Eq)]
pub struct TestCaseEnv {
    inner: Arc<TestCaseInner>,
}

impl Default for TestCaseEnv {
    fn default() -> Self {
        Self {
            inner: Arc::new(TestCaseInner {
                fetch_handler: Box::new(default_fetch_handler),
                requests: Default::default(),
                storage: Default::default(),
                events: Default::default(),
                states: Default::default(),
                now: Utc::now(),
            }),
        }
    }
}

impl TestCaseEnv {
    pub fn new(
        fetcher: impl Into<Option<FetchHandler>>,
        now: impl Into<Option<DateTime<Utc>>>,
    ) -> Self {
        let fetcher: Option<FetchHandler> = fetcher.into();
        let now_datetime = now.into();
        Self {
            inner: Arc::new(TestCaseInner {
                fetch_handler: fetcher.unwrap_or(Box::new(default_fetch_handler)),
                requests: Default::default(),
                storage: Default::default(),
                events: Default::default(),
                states: Default::default(),
                now: now_datetime.unwrap_or(Utc::now()),
            }),
        }
    }
}

// #[derive(Debug, PartialEq, Eq)]
struct TestCaseInner {
    pub fetch_handler: FetchHandler,
    pub requests: Vec<Request>,
    pub storage: BTreeMap<String, String>,
    pub events: Vec<Box<dyn Any + Send + Sync + 'static>>,
    pub states: Vec<Box<dyn Any + Send + Sync + 'static>>,
    pub now: DateTime<Utc>,
}

#[cfg(test)]
mod test {
    use std::{panic, thread::LocalKey};

    use chrono::{TimeZone, Utc};

    use super::*;

    fn setup<'a>() -> LockResult<MutexGuard<'static, ()>> {
        TestEnv2::reset()
    }

    /// any additional teardown actions
    fn teardown() {}
    fn run_test<T>(test: T) -> ()
    where
        // with thread_local
        T: FnOnce(&'static TestEnv2) -> () + panic::UnwindSafe,
    {
        let env_guard = setup().expect("Should lock the environment");

        let result = panic::catch_unwind(|| test(&*ENV));

        // test has finished, drop the Mutex guard
        drop(env_guard);
        teardown();

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn test_single_test_case() {
        run_test(|env| {
            env.set_now(Utc.with_ymd_and_hms(2020, 6, 27, 14, 20, 5).unwrap());

            let env_datetime = call_env_function::<TestEnv2>();
            let expected = Utc.with_ymd_and_hms(2020, 6, 27, 14, 20, 5).unwrap();
            assert_eq!(expected, env_datetime);
        });
    }

    fn call_env_function<E: Env + 'static>() -> DateTime<Utc> {
        E::now()
    }
}
