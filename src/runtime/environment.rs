use crate::addon_transport::{AddonHTTPTransport, AddonTransport};
use chrono::{DateTime, Utc};
use futures::future::{Future, LocalBoxFuture};
use http::Request;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

#[derive(Debug)]
pub enum EnvError {
    StorageUnavailable,
    StorageSchemaVersionDowngrade(usize, usize),
    Fetch(String),
    AddonTransport(String),
    Serde(serde_json::error::Error),
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::error::Error> for EnvError {
    fn from(error: serde_json::error::Error) -> Self {
        EnvError::Serde(error)
    }
}

pub type EnvFuture<T> = LocalBoxFuture<'static, Result<T, EnvError>>;

pub trait Environment {
    fn fetch<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: Serialize + 'static,
        for<'de> OUT: Deserialize<'de> + 'static;
    fn exec<F>(future: F)
    where
        F: Future<Output = ()> + 'static;
    fn now() -> DateTime<Utc>;
    fn get_storage<T>(key: &str) -> EnvFuture<Option<T>>
    where
        for<'de> T: Deserialize<'de> + 'static;
    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn addon_transport(transport_url: &Url) -> Box<dyn AddonTransport>
    where
        Self: Sized + 'static,
    {
        Box::new(AddonHTTPTransport::<Self>::new(transport_url))
    }
}
